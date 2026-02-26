import { parseArgs } from 'node:util'
import { readFile, mkdir, readdir, rename } from 'node:fs/promises'
import { join, dirname, basename } from 'node:path'
import chokidar from 'chokidar'
import { GoogleGenAI } from '@google/genai'
import { z } from 'zod'
import configJs from './config.js'

await process.loadEnvFile('.env')
const { positionals } = parseArgs({ allowPositionals: true })
const config = configJs[positionals[0]]

const schema = config.schema

const ai = new GoogleGenAI({ apiKey: process.env.GEMINI_API_KEY })

await mkdir(join(config.path, 'backup')).catch(() => {})
const oldFiles = await readdir(config.path)

for await (const file of oldFiles) {
  if (file.endsWith('.jpg')) {
    const fullpath = join(config.path, file)
    await rename(fullpath, join(dirname(fullpath), 'backup', basename(fullpath)))
  }
}

const watcher = chokidar.watch(config.path, {
  depth: 0,
  ignored: (path, stats) => stats?.isFile() && !path.endsWith('.jpg'),
})
console.log(`${config.path}を監視中`)

watcher.on('add', async (path) => {
  console.log('翻訳開始')
  await fetch(process.env.DISCORD_WEBHOOK_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username: 'Game Lens',
      content: '翻訳処理中…',
      avatar_url: config.avatar_url,
    }),
  })

  const base64ImageFile = await readFile(path, { encoding: 'base64' })

  const response = await ai.models.generateContent({
    model: 'gemini-2.5-flash',
    config: {
      responseMimeType: 'application/json',
      responseJsonSchema: z.toJSONSchema(schema),
      thinkingConfig: {
        thinkingBudget: 600,
      },
    },
    contents: [
      {
        inlineData: { mimeType: 'image/jpeg', data: base64ImageFile },
      },
      { text: config.prompt },
    ],
  })
  const result = schema.parse(JSON.parse(response.text))
  console.log(response.usageMetadata)

  await fetch(process.env.DISCORD_WEBHOOK_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username: result.speaker,
      content: result.lines.join('\n'),
      avatar_url: config.avatar_url,
    }),
  })
  await fetch(process.env.DISCORD_WEBHOOK_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      username: '選択肢',
      content: result.player_options.join('\n'),
      avatar_url: config.avatar_url,
    }),
  })
})
