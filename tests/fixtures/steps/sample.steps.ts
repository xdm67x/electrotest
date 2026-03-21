export type StepContext = {
  appTitle(): Promise<string>;
};

export function registerSampleSteps(register: (pattern: string, handler: (ctx: StepContext, expected: string) => Promise<void>) => void) {
  register("the fixture window title should be {string}", async (ctx, expected) => {
    const actual = await ctx.appTitle();

    if (actual !== expected) {
      throw new Error(`expected fixture window title to be ${expected}, got ${actual}`);
    }
  });
}
