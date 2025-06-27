import random
from enum import Enum
import csv
from collections import Counter
from itertools import chain
import pprint

from statemachine import State
from statemachine import StateMachine

import click


class SimError(Exception):
    pass


class BusyError(SimError):
    pass


class FullError(SimError):
    pass


class SimulationEntity:
    def tick(self) -> None:
        raise NotImplementedError


class Component(Enum):
    A, B, C = range(3)

    def __repr__(self) -> str:
        return format(self.name)

class Cell:
    def peek(self) -> Component | None:
        return None

    def get(self) -> Component | None:
        return None

    def put(self, component: Component):
        pass

    def refresh(self):
        pass


class Belt(SimulationEntity):
    def __init__(self, cells: list[Cell]):
        self.cells = cells

    def refresh(self):
        for cell in self.cells:
            cell.refresh()

    def tick(self):
        self.refresh()
        ouputs = [cell.get() for cell in self.cells]
        self.refresh()
        for i in range(1, len(self.cells)):
            self.cells[i].put(ouputs[i - 1])
        self.refresh()


class SimpleWorker(SimulationEntity):
    class SimpleWorkerStateMachine(StateMachine):
        # States
        Empty = State(initial=True)
        A = State()
        B = State()
        AB = State("AB")
        Work1 = State("Working...")
        Work2 = State("Working...")
        Work3 = State("Working...")
        C = State()

        # Transitions
        cell_a = (
            Empty.to(A, cond="cell_not_busy", on="take")
            | B.to(AB, cond="cell_not_busy", on="take")
            | AB.to(Work1)
            | Work1.to(Work2)
            | Work2.to(Work3)
            | Work3.to(C)
            | A.to.itself()
            | C.to.itself()
        )
        cell_b = (
            Empty.to(B, cond="cell_not_busy", on="take")
            | A.to(AB, cond="cell_not_busy", on="take")
            | AB.to(Work1)
            | Work1.to(Work2)
            | Work2.to(Work3)
            | Work3.to(C)
            | B.to.itself()
            | C.to.itself()
        )
        cell_empty = (
            C.to(Empty, cond="cell_not_busy", on="putC")
            | C.to.itself()
            | AB.to(Work1)
            | Work1.to(Work2)
            | Work2.to(Work3)
            | Work3.to(C)
            | Empty.to.itself()
            | A.to.itself()
            | B.to.itself()
            | AB.to.itself()
        )

        def __init__(self, cell):
            self.cell = cell
            super().__init__()

        def cell_not_busy(self):
            return not self.cell.busy

        def take(self):
            return self.cell.get()

        def putC(self):
            self.cell.put(Component.C)

    def __init__(self, cell):
        self.cell = cell
        self.machine = SimpleWorker.SimpleWorkerStateMachine(cell)

    def tick(self):
        match self.cell.peek():
            case Component.A:
                self.machine.cell_a()
            case Component.B:
                self.machine.cell_b()
            case None:
                self.machine.cell_empty()

    def inventory(self) -> list[Component]:
        match self.machine.current_state:
            case self.machine.Empty:
                return [None]
            case self.machine.A:
                return [Component.A]
            case self.machine.B:
                return [Component.B]
            case self.machine.AB:
                return [Component.A, Component.B]
            case self.machine.Work1:
                return [Component.A, Component.B]
            case self.machine.Work2:
                return [Component.A, Component.B]
            case self.machine.Work3:
                return [Component.A, Component.B]
            case self.machine.C:
                return [Component.C]
            case _:
                raise ValueError(f"Invalid state: {self.machine.current_state}")




class WorkerCell(Cell):
    def __init__(self):
        self.inventory = None
        self.busy = False

    def peek(self) -> Component | None:
        return self.inventory

    def get(self) -> Component | None:
        if self.busy:
            raise BusyError("Cell is busy")
        inventory = self.inventory
        self.inventory = None
        self.busy = True
        return inventory

    def put(self, component: Component):
        if self.busy:
            raise BusyError("Cell is busy")
        if self.inventory is not None:
            raise FullError("Cell is full")
        self.inventory = component
        self.busy = True

    def refresh(self):
        self.busy = False


class RandomSource(Cell):
    def __init__(self, choices):
        self.choices = choices
        self.tally = Counter()

    def get(self) -> Component | None:
        component = random.choice(self.choices)
        self.tally[component] += 1
        return component


class TallySink(Cell):
    def __init__(self):
        self.tally = Counter()

    def put(self, component: Component):
        self.tally[component] += 1




class Simulation:
    def __init__(self, ticks, belt_length, workers):
        self.source = RandomSource([Component.A, Component.B, None])
        self.sink = TallySink()

        self.workers = []
        cells = []

        for i in range(belt_length):
            cell = WorkerCell()
            cells.append(cell)
            for j in range(workers):
                self.workers.append(SimpleWorker(cell))

        self.belt = Belt([self.source] + cells + [self.sink])

    def tick(self):
        self.belt.tick()
        for worker in self.workers:
            worker.tick()

    def stats(self):
        return {
            "input": self.source.tally,
            "output": self.sink.tally,
            "belt": Counter([cell.peek() for cell in self.belt.cells]),
            "workers": Counter(
                chain.from_iterable([worker.inventory() for worker in self.workers])
            ),
        }

    def show(self):
        print(f"{self.sink.tally}")


@click.group()
def main():
    pass


@main.command()
@click.option(
    "--output-file",
    default=None,
    type=click.File("w"),
    help="Save simulation data to CSV file",
)
@click.option("-t", "--ticks", default=100, help="Simulation length in ticks")
@click.option("-v", "--verbose", default=True, help="Show simulation progress")
@click.option("-s", "--seed", default=None, help="Random seed")
@click.option("-b", "--belt-length", default=3, help="Length of belt")
@click.option("-w", "--workers", default=2, help="Number of workers per work cell")
def run(ticks=100, verbose=False, seed=None, belt_length=3, workers=2, **kwargs):
    """Runs the factory simulation."""
    
    if belt_length < 2:
        raise ValueError("Belt length must be at least 2")
    if workers < 1:
        raise ValueError("Number of workers must be at least 1")
    if ticks < 1:
        raise ValueError("Ticks must be at least 1")
    
    if seed is not None:
        random.seed(seed)
    
    sim = Simulation(ticks=ticks, belt_length=belt_length, workers=workers)

    writer = None
    if kwargs["output_file"]:
        writer = csv.writer(kwargs["output_file"])
        writer.writerow(["A", "B", "C"])

    for _ in range(ticks):
        sim.tick()
        if verbose:
            sim.show()
        if writer:
            writer.writerow(
                [
                    sim.sink.tally.get(Component.A, 0),
                    sim.sink.tally.get(Component.B, 0),
                    sim.sink.tally.get(Component.C, 0),
                ]
            )
    if verbose:
        stats = sim.stats()
        print()
        print("STATS:")
        pprint.pprint(stats)
        print()
        print("INVARIANTS:")
        output = stats["output"].get(Component.C, 0) * 2
        output += stats["output"].get(Component.A, 0)
        output += stats["output"].get(Component.B, 0)
        output += stats["workers"].get(Component.A, 0)
        output += stats["workers"].get(Component.B, 0)
        output += stats["workers"].get(Component.C, 0) * 2
        output += stats["belt"].get(Component.A, 0)
        output += stats["belt"].get(Component.B, 0)
        output += stats["belt"].get(Component.C, 0) * 2

        input = stats["input"].get(Component.A, 0)
        input += stats["input"].get(Component.B, 0)

        symbol = "✅" if output == input else "❌"
        formula = "2Po+Ao+Bo+Aw+Bw+2Pw+Ab+Bb+2Pb = Ai+Bi"
        details = "" if output == input else f"; {output} != {input}"

        print(f"{symbol} {formula}{details}")


@main.command()
@click.argument("FILE", type=click.File("w"))
def show_worker_statemachine(**kwargs):
    """Save state machine diagram for worker in FILE."""
    from statemachine.contrib.diagram import DotGraphMachine

    graph = DotGraphMachine(SimpleWorker.SimpleWorkerStateMachine)
    dot = graph()
    dot.write_png(kwargs["file"].name)


if __name__ == "__main__":
    main()
