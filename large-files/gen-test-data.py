import random


def main():
    size = 2000

    with open("test.csv", "w") as tf:
        for row in range(size):
            for column in range(size):
                pick_nan = random.random() < 0.2
                if pick_nan:
                    tf.write("NaN")
                else:
                    tf.write(str(random.random()))
                if column != (size - 1):
                    tf.write(",")
            tf.write("\n")


if __name__ == "__main__":
    main()
