import { join } from 'node:path'
import { z } from 'zod'

const config = {
  ddo: {
    name: 'Dungeons and Dragons Online',
    path: join(process.env['USERPROFILE'], 'Documents', 'Dungeons and Dragons Online'),
    prompt: '画像中の黒い背景のダイアログの会話文を日本語に翻訳してください。',
    schema: z.object({
      speaker: z.string(),
      lines: z.array(z.string()),
      player_options: z.array(z.string()),
    }),
    avatar_url:
      'https://images-ext-1.discordapp.net/external/SfHNcPgzdvapQgglxxfodbDxqC3Z7bTNGdrfsb87FBE/%3Fv%3D1482477394/https/www.ddo.com/images/global/header/ddo-logo-small.png?format=webp&quality=lossless',
  },
}

export default config
