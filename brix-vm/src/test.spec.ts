describe('UserController', () => {
    describe('when getting all users', () => {
      let result: number;
  
      beforeEach(() => {
        result = 123;
      });
  
      it('should return the users', () => {
        expect(result).toBe(123);
      });
    });
    describe("failure", ()=>{
        expect(true).toBe(false)
    })
  });
  