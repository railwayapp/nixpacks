import { Badge } from '@/components/Badge'
import { Background } from '@/components/Background'
import Link from 'next/link'
import React from 'react'

const Page = () => {
  return (
    <>
      <main>
        <article>
          <Badge />
          <h1>Payload 3.0</h1>
          <p>
            This BETA is rapidly evolving, you can report any bugs against{' '}
            <Link href="https://github.com/payloadcms/payload-3.0-demo/issues" target="_blank">
              the repo
            </Link>{' '}
            or in the{' '}
            <Link
              href="https://discord.com/channels/967097582721572934/1215659716538273832"
              target="_blank"
            >
              dedicated channel in Discord
            </Link>
            . Payload is running at <Link href="/admin">/admin</Link>. An example of a custom route
            running the Local API can be found at <Link href="/my-route">/my-route</Link>.
          </p>
          <p>You can use the Local API in your server components like this:</p>
        </article>
        <div className="codeBlock">
          <pre>
            <code>
              {`import { getPayloadHMR } from '@payloadcms/next/utilities'
import configPromise from '@payload-config'
const payload = await getPayloadHMR({ config: configPromise })

const data = await payload.find({
  collection: 'posts',
})

return <Posts data={data} />
`}
            </code>
          </pre>
        </div>
      </main>
      <Background />
    </>
  )
}

export default Page
