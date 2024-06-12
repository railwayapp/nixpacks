import React from 'react'
import { getPayloadHMR } from '@payloadcms/next/utilities'
import configPromise from '@payload-config'

const Example: React.FC = async () => {
  const payload = await getPayloadHMR({ config: configPromise })
  const url = payload.getAdminURL()
  return <div>The admin panel is running at: {url}</div>
}

export default Example
