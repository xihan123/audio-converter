/**
 * 批量下载工具函数
 * 将多个 WAV 文件打包为 ZIP 文件下载
 */

import JSZip from 'jszip'

interface DownloadFile {
    name: string
    url: string
}

/**
 * 批量下载文件（打包为 ZIP）
 */
export async function downloadFilesAsZip(
    files: DownloadFile[],
    zipFileName: string = 'audio-files.zip',
    onProgress?: (progress: number) => void
): Promise<void> {
    if (files.length === 0) {
        throw new Error('没有可下载的文件')
    }

    const zip = new JSZip()

    // 添加文件到 ZIP
    for (let i = 0; i < files.length; i++) {
        const file = files[i]

        try {
            // 从 URL 获取 Blob 数据
            const response = await fetch(file.url)
            const blob = await response.blob()

            // 确保文件名唯一（如果有重名，添加序号）
            let fileName = file.name
            const existingFile = zip.file(fileName)
            if (existingFile) {
                const nameParts = fileName.split('.')
                const ext = nameParts.pop()
                const baseName = nameParts.join('.')
                fileName = `${baseName}_${i + 1}.${ext}`
            }

            // 添加到 ZIP
            zip.file(fileName, blob)

            // 更新进度
            if (onProgress) {
                const progress = ((i + 1) / files.length) * 80 // 0-80% 用于添加文件
                onProgress(progress)
            }
        } catch (error) {
            console.error(`无法下载文件 ${file.name}:`, error)
        }
    }

    // 生成 ZIP 文件
    if (onProgress) {
        onProgress(85)
    }

    const zipBlob = await zip.generateAsync(
        {type: 'blob'},
        (metadata) => {
            // 压缩进度：85-100%
            if (onProgress) {
                const progress = 85 + (metadata.percent * 0.15)
                onProgress(progress)
            }
        }
    )

    // 下载 ZIP 文件
    const url = URL.createObjectURL(zipBlob)
    const a = document.createElement('a')
    a.href = url
    a.download = zipFileName
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    if (onProgress) {
        onProgress(100)
    }
}

/**
 * 生成带时间戳的 ZIP 文件名
 */
export function generateZipFileName(prefix: string = 'audio-files'): string {
    const now = new Date()
    const dateStr = now.toISOString().slice(0, 10).replace(/-/g, '')
    const timeStr = now.toTimeString().slice(0, 8).replace(/:/g, '')
    return `${prefix}_${dateStr}_${timeStr}.zip`
}
