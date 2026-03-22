import { handleRequest } from './engine.js'
import type { Request } from './protocol.js'

declare const process: {
  stdin: AsyncIterable<string> & { setEncoding(encoding: string): void }
  stdout: { write(chunk: string): void }
}

async function* readLines(input: AsyncIterable<string>): AsyncIterable<string> {
  let buffer = ''

  for await (const chunk of input) {
    buffer += chunk

    let newlineIndex = buffer.indexOf('\n')
    while (newlineIndex >= 0) {
      const line = buffer.slice(0, newlineIndex).replace(/\r$/, '')
      buffer = buffer.slice(newlineIndex + 1)
      yield line
      newlineIndex = buffer.indexOf('\n')
    }
  }

  if (buffer.length > 0) {
    yield buffer.replace(/\r$/, '')
  }
}

process.stdin.setEncoding('utf8')

for await (const line of readLines(process.stdin)) {
  const request = JSON.parse(line) as Request
  const response = await handleRequest(request)
  process.stdout.write(JSON.stringify(response) + '\n')
}
