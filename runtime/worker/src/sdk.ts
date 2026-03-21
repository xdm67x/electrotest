export type StepContext = {
  appTitle(): Promise<string>;
  log(message: string): void;
};

export type StepHandler = (
  context: StepContext,
  ...args: string[]
) => Promise<void> | void;

export type RegisteredStep = {
  pattern: RegExp | string;
  handler: StepHandler;
};

export type StepSdk = {
  defineStep(pattern: RegExp | string, handler: StepHandler): RegisteredStep;
};

export function defineStep(
  pattern: RegExp | string,
  handler: StepHandler,
): RegisteredStep {
  return { pattern, handler };
}
