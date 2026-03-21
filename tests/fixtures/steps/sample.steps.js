export function registerSampleSteps(register) {
  register("the fixture js step should run", async (ctx) => {
    ctx.log("custom js step executed");
  });
}
