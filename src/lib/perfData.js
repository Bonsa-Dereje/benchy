// Ported from the marketplace site's index.html so the desktop app's
// department/app picker looks and behaves the same way.

export const DEPT_ICONS = {
  'video-editor': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="2" y="4" width="20" height="16" rx="2"/><polygon points="10,9 10,15 15,12" fill="currentColor" stroke="none"/><line x1="2" y1="8" x2="22" y2="8"/></svg>`,
  graphics: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="3"/><line x1="12" y1="3" x2="12" y2="9"/><line x1="12" y1="15" x2="12" y2="21"/><line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/></svg>`,
  '3d': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M12 2l9 5v10l-9 5-9-5V7z"/><polyline points="12 22 12 12"/><path d="M3.27 6.96L12 12.01l8.73-5.05M3.27 17.04L12 12"/></svg>`,
  writer: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>`,
  cs: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/><line x1="15" y1="4" x2="9" y2="20"/></svg>`,
  'local-ai': `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="3" y="6" width="18" height="12" rx="2"/><circle cx="7" cy="12" r="1.5" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none"/><circle cx="17" cy="12" r="1.5" fill="currentColor" stroke="none"/><path d="M7 6V4M12 6V4M17 6V4M7 18v2M12 18v2M17 18v2"/></svg>`,
  gaming: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M6 12h4M8 10v4"/><circle cx="15" cy="11" r="1" fill="currentColor" stroke="none"/><circle cx="17" cy="13" r="1" fill="currentColor" stroke="none"/><path d="M3 8a2 2 0 012-2h14a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8z"/></svg>`,
  data: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>`,
  engineering: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><circle cx="12" cy="12" r="3"/><path d="M19.07 4.93l-1.41 1.41M5.34 18.66l-1.41 1.41M20 12h-2M6 12H4M19.07 19.07l-1.41-1.41M5.34 5.34L3.93 3.93M12 20v-2M12 6V4"/></svg>`,
}

// Icons for the specs readout + status states. Same stroke convention as
// DEPT_ICONS above (currentColor, 1.7 stroke) so everything reads as one set.
export const SPEC_ICONS = {
  host: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="3" y="4" width="18" height="12" rx="1.5"/><line x1="8" y1="20" x2="16" y2="20"/><line x1="12" y1="16" x2="12" y2="20"/></svg>`,
  cpu: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="7" y="7" width="10" height="10" rx="1.2"/><rect x="10.3" y="10.3" width="3.4" height="3.4"/><path d="M9 2v3M12 2v3M15 2v3M9 19v3M12 19v3M15 19v3M2 9h3M2 12h3M2 15h3M19 9h3M19 12h3M19 15h3"/></svg>`,
  ram: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="3" y="5" width="18" height="9" rx="1.2"/><path d="M6 14v4M9.4 14v4M12.8 14v4M16.2 14v4M19 14v4"/></svg>`,
  gpu: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="2" y="7" width="20" height="10" rx="1.2"/><circle cx="15.5" cy="12" r="2.6"/><path d="M5 7V5M9 7V5"/><line x1="5" y1="17" x2="5" y2="19"/></svg>`,
  storage: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="3" y="4" width="18" height="16" rx="1.5"/><line x1="3" y1="14" x2="21" y2="14"/><circle cx="7.3" cy="17" r="1" fill="currentColor" stroke="none"/><line x1="11" y1="17" x2="17" y2="17"/></svg>`,
}

// Small status/utility icons used around loading + fallback states.
export const UI_ICONS = {
  spark: `<svg viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M12 2.5l1.9 6.6 6.6 1.9-6.6 1.9L12 19.5l-1.9-6.6L3.5 11l6.6-1.9z"/></svg>`,
  alert: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><circle cx="12" cy="12" r="9"/><line x1="12" y1="7.5" x2="12" y2="13"/><circle cx="12" cy="16.3" r="0.9" fill="currentColor" stroke="none"/></svg>`,
  check: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 12.5 9.5 18 20 6"/></svg>`,
}

export const APP_ICONS = {
  'Adobe Photoshop': 'https://www.google.com/s2/favicons?sz=32&domain=adobe.com',
  Figma: 'https://www.google.com/s2/favicons?sz=32&domain=figma.com',
  'Adobe Illustrator': 'https://www.google.com/s2/favicons?sz=32&domain=adobe.com',
  'Premiere Pro': 'https://www.google.com/s2/favicons?sz=32&domain=adobe.com',
  'After Effects': 'https://www.google.com/s2/favicons?sz=32&domain=adobe.com',
  Blender: 'https://www.google.com/s2/favicons?sz=32&domain=blender.org',
  'DaVinci Resolve': 'https://www.google.com/s2/favicons?sz=32&domain=blackmagicdesign.com',
  Lightroom: 'https://www.google.com/s2/favicons?sz=32&domain=adobe.com',
  'VS Code': 'https://www.google.com/s2/favicons?sz=32&domain=code.visualstudio.com',
  Docker: 'https://www.google.com/s2/favicons?sz=32&domain=docker.com',
  'Android Studio': 'https://www.google.com/s2/favicons?sz=32&domain=developer.android.com',
  Xcode: 'https://www.google.com/s2/favicons?sz=32&domain=developer.apple.com',
  'JetBrains IDEs': 'https://www.google.com/s2/favicons?sz=32&domain=jetbrains.com',
  Postman: 'https://www.google.com/s2/favicons?sz=32&domain=postman.com',
  'Jupyter Notebook': 'https://www.google.com/s2/favicons?sz=32&domain=jupyter.org',
  'Power BI': 'https://www.google.com/s2/favicons?sz=32&domain=powerbi.microsoft.com',
  'Excel (Heavy)': 'https://www.google.com/s2/favicons?sz=32&domain=microsoft.com',
  'Python / R (Data)': 'https://www.google.com/s2/favicons?sz=32&domain=python.org',
  'Word / Google Docs': 'https://www.google.com/s2/favicons?sz=32&domain=docs.google.com',
  Notion: 'https://www.google.com/s2/favicons?sz=32&domain=notion.so',
  Grammarly: 'https://www.google.com/s2/favicons?sz=32&domain=grammarly.com',
  'ChatGPT / Claude': 'https://www.google.com/s2/favicons?sz=32&domain=openai.com',
  AutoCAD: 'https://www.google.com/s2/favicons?sz=32&domain=autodesk.com',
  SolidWorks: 'https://www.google.com/s2/favicons?sz=32&domain=solidworks.com',
  'Fusion 360': 'https://www.google.com/s2/favicons?sz=32&domain=autodesk.com',
  MATLAB: 'https://www.google.com/s2/favicons?sz=32&domain=mathworks.com',
  Ollama: 'https://www.google.com/s2/favicons?sz=32&domain=ollama.ai',
  'LM Studio': 'https://www.google.com/s2/favicons?sz=32&domain=lmstudio.ai',
  'Stable Diffusion': 'https://www.google.com/s2/favicons?sz=32&domain=stability.ai',
  ComfyUI: 'https://www.google.com/s2/favicons?sz=32&domain=comfy.org',
  'Whisper (local)': 'https://www.google.com/s2/favicons?sz=32&domain=openai.com',
  Steam: 'https://www.google.com/s2/favicons?sz=32&domain=store.steampowered.com',
  'Epic Games': 'https://www.google.com/s2/favicons?sz=32&domain=epicgames.com',
  Discord: 'https://www.google.com/s2/favicons?sz=32&domain=discord.com',
  'Unreal Engine': 'https://www.google.com/s2/favicons?sz=32&domain=unrealengine.com',
  Unity: 'https://www.google.com/s2/favicons?sz=32&domain=unity.com',
  'Microsoft Office': 'https://www.google.com/s2/favicons?sz=32&domain=microsoft.com',
  'Google Chrome': 'https://www.google.com/s2/favicons?sz=32&domain=google.com',
  Zoom: 'https://www.google.com/s2/favicons?sz=32&domain=zoom.us',
}

export const DEPARTMENTS = [
  { id: 'video-editor', label: 'Video Editor', icon: 'video-editor', apps: ['DaVinci Resolve', 'Premiere Pro', 'After Effects', 'Blender'] },
  { id: 'graphics', label: 'Graphics Designer', icon: 'graphics', apps: ['Adobe Photoshop', 'Adobe Illustrator', 'Figma', 'Lightroom'] },
  { id: '3d', label: '3D Modeler', icon: '3d', apps: ['Blender', 'Fusion 360', 'Unreal Engine', 'Unity'] },
  { id: 'writer', label: 'Writer / Student', icon: 'writer', apps: ['Word / Google Docs', 'Notion', 'Grammarly', 'ChatGPT / Claude'] },
  { id: 'engineering', label: 'Engineering / CAD', icon: 'engineering', apps: ['AutoCAD', 'SolidWorks', 'Fusion 360', 'MATLAB'] },
  { id: 'cs', label: 'Developer', icon: 'cs', apps: ['VS Code', 'Docker', 'JetBrains IDEs', 'Android Studio'] },
  { id: 'local-ai', label: 'Local AI Workloads', icon: 'local-ai', apps: ['Ollama', 'LM Studio', 'Stable Diffusion', 'ComfyUI'] },
  { id: 'gaming', label: 'Gaming', icon: 'gaming', apps: ['Steam', 'Epic Games', 'Unreal Engine', 'Discord'] },
  { id: 'data', label: 'Data & Analytics', icon: 'data', apps: ['Power BI', 'Excel (Heavy)', 'Python / R (Data)', 'Jupyter Notebook'] },
]

// The "general readiness" panel shown right after specs load, before the
// user picks a department. Kept short on purpose — it's a snapshot, not
// the full app-specific report.
export const GENERAL_CATEGORIES = [
  'Everyday Use & Browsing',
  'Study, Reading & Office Work',
  'Video Editing & Rendering',
  '3D Modeling & CAD',
  'Running LLMs Locally',
  'Gaming',
]