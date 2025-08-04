import { describe, it, expect, beforeEach } from 'vitest'
import {
  validateDroppedFiles,
  getFileExtension,
  isAudioFile,
  isVideoFile,
  getFileTypeDescription,
  formatFileSize,
  addDropFeedback,
  removeDropFeedback
} from '../file-drop'

describe('file-drop utilities', () => {
  describe('validateDroppedFiles', () => {
    it('should validate empty file list', () => {
      const result = validateDroppedFiles([])
      expect(result.valid).toBe(false)
      expect(result.errors).toContain('No files selected')
    })

    it('should validate too many files', () => {
      const files = Array(101).fill(null).map((_, i) => 
        new File(['content'], `file${i}.txt`, { type: 'text/plain' })
      )
      
      const result = validateDroppedFiles(files)
      expect(result.valid).toBe(false)
      expect(result.errors[0]).toContain('Too many files: 101')
    })

    it('should warn about large files', () => {
      const largeFile = new File(['content'], 'large.wav')
      Object.defineProperty(largeFile, 'size', { value: 1024 * 1024 * 1024 + 1 })
      
      const result = validateDroppedFiles([largeFile])
      expect(result.valid).toBe(true)
      expect(result.warnings[0]).toContain('Large file: large.wav')
    })

    it('should warn about unsafe file names', () => {
      const unsafeFile = new File(['content'], '../unsafe.txt')
      
      const result = validateDroppedFiles([unsafeFile])
      expect(result.valid).toBe(true)
      expect(result.warnings[0]).toContain('Potentially unsafe file name')
    })

    it('should validate normal files', () => {
      const files = [
        new File(['content'], 'test.wav', { type: 'audio/wav' }),
        new File(['content'], 'test.mp3', { type: 'audio/mp3' })
      ]
      
      const result = validateDroppedFiles(files)
      expect(result.valid).toBe(true)
      expect(result.errors).toHaveLength(0)
    })
  })

  describe('getFileExtension', () => {
    it('should extract file extension', () => {
      expect(getFileExtension('test.wav')).toBe('wav')
      expect(getFileExtension('path/to/file.MP3')).toBe('mp3')
      expect(getFileExtension('file.tar.gz')).toBe('gz')
    })

    it('should handle files without extension', () => {
      expect(getFileExtension('README')).toBe('')
      expect(getFileExtension('path/to/file')).toBe('')
    })

    it('should handle edge cases', () => {
      expect(getFileExtension('.hidden')).toBe('hidden') // .hidden files have extension 'hidden'
      expect(getFileExtension('file.')).toBe('')
      expect(getFileExtension('')).toBe('')
    })
  })

  describe('isAudioFile', () => {
    it('should identify audio files', () => {
      expect(isAudioFile('test.wav')).toBe(true)
      expect(isAudioFile('music.MP3')).toBe(true)
      expect(isAudioFile('audio.flac')).toBe(true)
      expect(isAudioFile('sound.aiff')).toBe(true)
    })

    it('should reject non-audio files', () => {
      expect(isAudioFile('video.mp4')).toBe(false)
      expect(isAudioFile('document.txt')).toBe(false)
      expect(isAudioFile('image.jpg')).toBe(false)
    })
  })

  describe('isVideoFile', () => {
    it('should identify video files', () => {
      expect(isVideoFile('movie.mp4')).toBe(true)
      expect(isVideoFile('clip.AVI')).toBe(true)
      expect(isVideoFile('video.mkv')).toBe(true)
      expect(isVideoFile('film.mov')).toBe(true)
    })

    it('should reject non-video files', () => {
      expect(isVideoFile('audio.wav')).toBe(false)
      expect(isVideoFile('document.txt')).toBe(false)
      expect(isVideoFile('image.jpg')).toBe(false)
    })
  })

  describe('getFileTypeDescription', () => {
    it('should describe audio files', () => {
      expect(getFileTypeDescription('test.wav')).toBe('Audio file (WAV)')
      expect(getFileTypeDescription('music.mp3')).toBe('Audio file (MP3)')
    })

    it('should describe video files', () => {
      expect(getFileTypeDescription('movie.mp4')).toBe('Video file (MP4)')
      expect(getFileTypeDescription('clip.avi')).toBe('Video file (AVI)')
    })

    it('should describe common file types', () => {
      expect(getFileTypeDescription('document.txt')).toBe('Text file')
      expect(getFileTypeDescription('data.json')).toBe('JSON file')
      expect(getFileTypeDescription('image.jpg')).toBe('JPEG image')
    })

    it('should handle unknown file types', () => {
      expect(getFileTypeDescription('file.xyz')).toBe('XYZ file')
      expect(getFileTypeDescription('noextension')).toBe('Unknown file type')
    })
  })

  describe('formatFileSize', () => {
    it('should format bytes', () => {
      expect(formatFileSize(0)).toBe('0 B')
      expect(formatFileSize(512)).toBe('512 B')
      expect(formatFileSize(1023)).toBe('1023 B')
    })

    it('should format kilobytes', () => {
      expect(formatFileSize(1024)).toBe('1 KB')
      expect(formatFileSize(1536)).toBe('1.5 KB')
    })

    it('should format megabytes', () => {
      expect(formatFileSize(1024 * 1024)).toBe('1 MB')
      expect(formatFileSize(1024 * 1024 * 2.5)).toBe('2.5 MB')
    })

    it('should format gigabytes', () => {
      expect(formatFileSize(1024 * 1024 * 1024)).toBe('1 GB')
      expect(formatFileSize(1024 * 1024 * 1024 * 1.5)).toBe('1.5 GB')
    })
  })

  describe('DOM feedback functions', () => {
    let element: HTMLElement

    beforeEach(() => {
      element = document.createElement('div')
    })

    describe('addDropFeedback', () => {
      it('should add valid drop target feedback', () => {
        addDropFeedback(element, true)
        
        expect(element.classList.contains('drag-over')).toBe(true)
        expect(element.classList.contains('valid-drop-target')).toBe(true)
        expect(element.style.backgroundColor).toBe('rgba(0, 255, 0, 0.1)')
        expect(element.style.border).toContain('2px dashed')
        expect(element.style.border).toContain('rgb(0, 255, 0)')
      })

      it('should add invalid drop target feedback', () => {
        addDropFeedback(element, false)
        
        expect(element.classList.contains('drag-over')).toBe(true)
        expect(element.classList.contains('invalid-drop-target')).toBe(true)
        expect(element.style.backgroundColor).toBe('rgba(255, 0, 0, 0.1)')
        expect(element.style.border).toContain('2px dashed')
        expect(element.style.border).toContain('rgb(255, 0, 0)')
      })
    })

    describe('removeDropFeedback', () => {
      it('should remove all feedback classes and styles', () => {
        // Add feedback first
        addDropFeedback(element, true)
        
        // Then remove it
        removeDropFeedback(element)
        
        expect(element.classList.contains('drag-over')).toBe(false)
        expect(element.classList.contains('valid-drop-target')).toBe(false)
        expect(element.classList.contains('invalid-drop-target')).toBe(false)
        expect(element.style.backgroundColor).toBe('')
        expect(element.style.border).toBe('')
      })
    })
  })
})