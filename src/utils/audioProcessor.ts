/**
 * 音频处理工具函数
 * 支持将各种格式的音频转换为 WAV 格式
 * 如果时长不足 30 秒，自动重复拼接
 */

interface ProcessResult {
    url: string
    size: number
    duration: number
}

const MIN_DURATION = 30 // 最小时长 30 秒

/**
 * 处理音频文件
 */
export async function processFile(
    file: File,
    onProgress: (progress: number) => void
): Promise<ProcessResult> {
    onProgress(10)

    // 1. 加载音频文件
    const audioBuffer = await loadAudioFile(file)
    onProgress(30)

    // 2. 检查时长，如果不足 30 秒则重复拼接
    const processedBuffer = await ensureMinDuration(audioBuffer)
    onProgress(60)

    // 3. 转换为 WAV 格式
    const wavBlob = await audioBufferToWav(processedBuffer)
    onProgress(90)

    // 4. 创建下载 URL
    const url = URL.createObjectURL(wavBlob)
    onProgress(100)

    return {
        url,
        size: wavBlob.size,
        duration: processedBuffer.duration
    }
}

/**
 * 加载音频文件到 AudioBuffer
 */
async function loadAudioFile(file: File): Promise<AudioBuffer> {
    const arrayBuffer = await file.arrayBuffer()
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()

    try {
        return await audioContext.decodeAudioData(arrayBuffer)
    } catch (error) {
        throw new Error('无法解码音频文件，请确保文件格式正确')
    }
}

/**
 * 确保音频时长至少为 30 秒
 * 如果不足则重复拼接
 */
async function ensureMinDuration(audioBuffer: AudioBuffer): Promise<AudioBuffer> {
    const duration = audioBuffer.duration

    if (duration >= MIN_DURATION) {
        return audioBuffer
    }

    // 计算需要重复的次数
    const repeatCount = Math.ceil(MIN_DURATION / duration)

    // 创建新的 AudioBuffer
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)()
    const newLength = Math.ceil(audioBuffer.sampleRate * MIN_DURATION)
    const newBuffer = audioContext.createBuffer(
        audioBuffer.numberOfChannels,
        newLength,
        audioBuffer.sampleRate
    )

    // 拷贝并重复音频数据
    for (let channel = 0; channel < audioBuffer.numberOfChannels; channel++) {
        const sourceData = audioBuffer.getChannelData(channel)
        const targetData = newBuffer.getChannelData(channel)

        for (let i = 0; i < repeatCount; i++) {
            const offset = i * sourceData.length
            for (let j = 0; j < sourceData.length && offset + j < targetData.length; j++) {
                targetData[offset + j] = sourceData[j]
            }
        }
    }

    return newBuffer
}

/**
 * 将 AudioBuffer 转换为 WAV 格式的 Blob
 */
async function audioBufferToWav(audioBuffer: AudioBuffer): Promise<Blob> {
    const numberOfChannels = audioBuffer.numberOfChannels
    const sampleRate = audioBuffer.sampleRate
    // const format = 1 // PCM - 注释掉未使用的变量
    const bitDepth = 16

    // 交织声道数据
    const interleaved = interleaveChannels(audioBuffer)

    // 转换为 16-bit PCM
    const pcmData = floatTo16BitPCM(interleaved)

    // 创建 WAV 文件头
    const wavHeader = createWavHeader(
        pcmData.length,
        numberOfChannels,
        sampleRate,
        bitDepth
    )

    // 合并头部和数据
    const wavData = new Uint8Array(wavHeader.length + pcmData.length)
    wavData.set(wavHeader, 0)
    wavData.set(pcmData, wavHeader.length)

    return new Blob([wavData], {type: 'audio/wav'})
}

/**
 * 交织多声道数据
 */
function interleaveChannels(audioBuffer: AudioBuffer): Float32Array {
    const numberOfChannels = audioBuffer.numberOfChannels
    const length = audioBuffer.length * numberOfChannels
    const result = new Float32Array(length)

    for (let channel = 0; channel < numberOfChannels; channel++) {
        const channelData = audioBuffer.getChannelData(channel)
        for (let i = 0; i < channelData.length; i++) {
            result[i * numberOfChannels + channel] = channelData[i]
        }
    }

    return result
}

/**
 * 将 Float32Array 转换为 16-bit PCM
 */
function floatTo16BitPCM(float32Array: Float32Array): Uint8Array {
    const buffer = new ArrayBuffer(float32Array.length * 2)
    const view = new DataView(buffer)

    for (let i = 0; i < float32Array.length; i++) {
        const s = Math.max(-1, Math.min(1, float32Array[i]))
        const val = s < 0 ? s * 0x8000 : s * 0x7fff
        view.setInt16(i * 2, val, true)
    }

    return new Uint8Array(buffer)
}

/**
 * 创建 WAV 文件头
 */
function createWavHeader(
    dataLength: number,
    numberOfChannels: number,
    sampleRate: number,
    bitDepth: number
): Uint8Array {
    const header = new ArrayBuffer(44)
    const view = new DataView(header)

    // RIFF identifier
    writeString(view, 0, 'RIFF')
    // file length
    view.setUint32(4, 36 + dataLength, true)
    // RIFF type
    writeString(view, 8, 'WAVE')
    // format chunk identifier
    writeString(view, 12, 'fmt ')
    // format chunk length
    view.setUint32(16, 16, true)
    // sample format (PCM)
    view.setUint16(20, 1, true)
    // channel count
    view.setUint16(22, numberOfChannels, true)
    // sample rate
    view.setUint32(24, sampleRate, true)
    // byte rate (sample rate * block align)
    view.setUint32(28, sampleRate * numberOfChannels * (bitDepth / 8), true)
    // block align (channel count * bytes per sample)
    view.setUint16(32, numberOfChannels * (bitDepth / 8), true)
    // bits per sample
    view.setUint16(34, bitDepth, true)
    // data chunk identifier
    writeString(view, 36, 'data')
    // data chunk length
    view.setUint32(40, dataLength, true)

    return new Uint8Array(header)
}

/**
 * 写入字符串到 DataView
 */
function writeString(view: DataView, offset: number, string: string): void {
    for (let i = 0; i < string.length; i++) {
        view.setUint8(offset + i, string.charCodeAt(i))
    }
}
