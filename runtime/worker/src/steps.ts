import { defineStep, type RegisteredStep, type StepSdk } from './sdk.js'

type LoadedModule = {
  default?: unknown
  registerSampleSteps?: (register: LegacyRegister) => void
}

type LegacyRegister = (pattern: string, handler: RegisteredStep['handler']) => void

type MatchedStep = {
  step: RegisteredStep
  args: string[]
}

export async function loadStepModules(stepPaths: string[]): Promise<RegisteredStep[]> {
  const compiled = await transpileTypescriptIfNeeded(stepPaths)
  const nodeUrl = await importNodeModule('node:url')
  const modules = await Promise.all(
    compiled.map((file) => import(nodeUrl.pathToFileURL(file).href as string)),
  )

  const steps: RegisteredStep[] = []
  for (const module of modules as LoadedModule[]) {
    steps.push(...(await extractRegisteredSteps(module)))
  }

  return steps
}

export function registeredStringPatterns(steps: RegisteredStep[]): string[] {
  return steps.flatMap((step) => (typeof step.pattern === 'string' ? [step.pattern] : []))
}

export function findMatchingStep(steps: RegisteredStep[], stepText: string): MatchedStep | null {
  const candidate = stripStepKeyword(stepText)

  for (const step of steps) {
    const args = matchPattern(step.pattern, candidate)
    if (args !== null) {
      return { step, args }
    }
  }

  return null
}

async function extractRegisteredSteps(module: LoadedModule): Promise<RegisteredStep[]> {
  if (typeof module.registerSampleSteps === 'function') {
    const steps: RegisteredStep[] = []
    module.registerSampleSteps((pattern, handler) => {
      steps.push(defineStep(pattern, handler))
    })
    return steps
  }

  const exported = module.default
  if (typeof exported === 'function') {
    return normalizeRegisteredSteps(await exported(createSdk()))
  }

  return normalizeRegisteredSteps(exported)
}

function createSdk(): StepSdk {
  return { defineStep }
}

function normalizeRegisteredSteps(value: unknown): RegisteredStep[] {
  if (Array.isArray(value)) {
    return value.filter(isRegisteredStep)
  }

  if (isRegisteredStep(value)) {
    return [value]
  }

  return []
}

function isRegisteredStep(value: unknown): value is RegisteredStep {
  return (
    typeof value === 'object' &&
    value !== null &&
    'pattern' in value &&
    'handler' in value &&
    typeof (value as { handler: unknown }).handler === 'function'
  )
}

async function transpileTypescriptIfNeeded(stepPaths: string[]): Promise<string[]> {
  const compiled: string[] = []

  for (const stepPath of stepPaths) {
    if (stepPath.endsWith('.ts')) {
      compiled.push(await transpileTypescript(stepPath))
    } else {
      compiled.push(stepPath)
    }
  }

  return compiled
}

async function transpileTypescript(stepPath: string): Promise<string> {
  const fs = await importNodeModule('node:fs/promises')
  const os = await importNodeModule('node:os')
  const path = await importNodeModule('node:path')
  const outputDir = await fs.mkdtemp(path.join(os.tmpdir(), 'electrotest-steps-'))
  await runCommand('tsc', [
    '--target',
    'ES2022',
    '--module',
    'NodeNext',
    '--moduleResolution',
    'NodeNext',
    '--outDir',
    outputDir,
    stepPath,
  ])

  return path.join(outputDir, path.basename(stepPath).replace(/\.ts$/, '.js'))
}

async function runCommand(command: string, args: string[]): Promise<void> {
  const childProcess = await importNodeModule('node:child_process')

  await new Promise<void>((resolve, reject) => {
    const child = childProcess.spawn(command, args)
    let stderr = ''

    if (child.stderr) {
      child.stderr.on('data', (chunk: { toString(): string }) => {
        stderr += chunk.toString()
      })
    }

    child.on('error', reject)
    child.on('exit', (code: number | null) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(stderr || `${command} exited with code ${code ?? 'unknown'}`))
      }
    })
  })
}

function stripStepKeyword(stepText: string): string {
  return stepText.replace(/^(Given|When|Then|And|But)\s+/, '')
}

function matchPattern(pattern: RegExp | string, stepText: string): string[] | null {
  if (typeof pattern === 'string') {
    return matchExpressionPattern(pattern, stepText)
  }

  const match = stepText.match(pattern)
  return match ? match.slice(1) : null
}

function matchExpressionPattern(pattern: string, stepText: string): string[] | null {
  const captures: string[] = []
  let remainingPattern = pattern
  let remainingStep = stepText

  while (remainingPattern.includes('{string}')) {
    const index = remainingPattern.indexOf('{string}')
    const prefix = remainingPattern.slice(0, index)

    if (!remainingStep.startsWith(prefix)) {
      return null
    }

    remainingStep = remainingStep.slice(prefix.length)
    if (!remainingStep.startsWith('"')) {
      return null
    }

    remainingStep = remainingStep.slice(1)
    const closingQuote = remainingStep.indexOf('"')
    if (closingQuote < 0) {
      return null
    }

    captures.push(remainingStep.slice(0, closingQuote))
    remainingStep = remainingStep.slice(closingQuote + 1)
    remainingPattern = remainingPattern.slice(index + '{string}'.length)
  }

  if (remainingStep !== remainingPattern) {
    return null
  }

  return captures
}

async function importNodeModule(specifier: string): Promise<any> {
  return new Function('specifier', 'return import(specifier);')(specifier) as Promise<any>
}
