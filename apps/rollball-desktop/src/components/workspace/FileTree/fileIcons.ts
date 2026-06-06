import type { LucideIcon } from "lucide-react";
import {
    File,
    FileText,
    FileCode,
    FileJson,
    FileType,
    FileImage,
    FileVideo,
    FileAudio,
    FileArchive,
    FileSpreadsheet,
    GitBranch,
    Package,
    Settings,
    Shield,
    Database,
    Terminal,
    Binary,
    Lock,
} from "lucide-react";

interface IconMapping {
    icon: LucideIcon;
    color: string; // Tailwind color class
}

/** Map file extension to icon and color */
const EXT_MAP: Record<string, IconMapping> = {
    // Rust
    rs: { icon: Binary, color: "text-orange-600 dark:text-orange-400" },
    toml: { icon: Settings, color: "text-zinc-500 dark:text-zinc-400" },
    lock: { icon: Lock, color: "text-zinc-400 dark:text-zinc-500" },

    // TypeScript / JavaScript
    ts: { icon: FileCode, color: "text-blue-600 dark:text-blue-400" },
    tsx: { icon: FileCode, color: "text-blue-600 dark:text-blue-400" },
    js: { icon: FileCode, color: "text-yellow-600 dark:text-yellow-400" },
    jsx: { icon: FileCode, color: "text-yellow-600 dark:text-yellow-400" },
    mjs: { icon: FileCode, color: "text-yellow-600 dark:text-yellow-400" },
    cjs: { icon: FileCode, color: "text-yellow-600 dark:text-yellow-400" },

    // Web
    html: { icon: FileCode, color: "text-orange-500 dark:text-orange-300" },
    css: { icon: FileCode, color: "text-blue-500 dark:text-blue-300" },
    scss: { icon: FileCode, color: "text-pink-500 dark:text-pink-300" },
    less: { icon: FileCode, color: "text-blue-400 dark:text-blue-300" },
    svg: { icon: FileImage, color: "text-yellow-500 dark:text-yellow-300" },

    // Data / Config
    json: { icon: FileJson, color: "text-yellow-600 dark:text-yellow-400" },
    yaml: { icon: FileJson, color: "text-red-500 dark:text-red-400" },
    yml: { icon: FileJson, color: "text-red-500 dark:text-red-400" },
    xml: { icon: FileCode, color: "text-orange-500 dark:text-orange-300" },

    // Markdown / Docs
    md: { icon: FileText, color: "text-zinc-500 dark:text-zinc-400" },
    mdx: { icon: FileText, color: "text-zinc-500 dark:text-zinc-400" },
    txt: { icon: FileText, color: "text-zinc-400 dark:text-zinc-500" },
    pdf: { icon: FileType, color: "text-red-500 dark:text-red-400" },

    // Shell
    sh: { icon: Terminal, color: "text-green-600 dark:text-green-400" },
    bash: { icon: Terminal, color: "text-green-600 dark:text-green-400" },
    ps1: { icon: Terminal, color: "text-blue-500 dark:text-blue-400" },
    bat: { icon: Terminal, color: "text-zinc-500 dark:text-zinc-400" },

    // Python
    py: { icon: FileCode, color: "text-green-500 dark:text-green-400" },
    pyi: { icon: FileCode, color: "text-green-500 dark:text-green-400" },

    // Go
    go: { icon: FileCode, color: "text-cyan-600 dark:text-cyan-400" },
    mod: { icon: Package, color: "text-cyan-500 dark:text-cyan-400" },
    sum: { icon: Lock, color: "text-zinc-400 dark:text-zinc-500" },

    // Images
    png: { icon: FileImage, color: "text-purple-500 dark:text-purple-400" },
    jpg: { icon: FileImage, color: "text-purple-500 dark:text-purple-400" },
    jpeg: { icon: FileImage, color: "text-purple-500 dark:text-purple-400" },
    gif: { icon: FileImage, color: "text-purple-500 dark:text-purple-400" },
    webp: { icon: FileImage, color: "text-purple-500 dark:text-purple-400" },
    ico: { icon: FileImage, color: "text-purple-500 dark:text-purple-400" },

    // Media
    mp4: { icon: FileVideo, color: "text-pink-500 dark:text-pink-400" },
    mov: { icon: FileVideo, color: "text-pink-500 dark:text-pink-400" },
    mp3: { icon: FileAudio, color: "text-pink-500 dark:text-pink-400" },
    wav: { icon: FileAudio, color: "text-pink-500 dark:text-pink-400" },

    // Archives
    zip: { icon: FileArchive, color: "text-yellow-600 dark:text-yellow-400" },
    tar: { icon: FileArchive, color: "text-yellow-600 dark:text-yellow-400" },
    gz: { icon: FileArchive, color: "text-yellow-600 dark:text-yellow-400" },

    // Spreadsheets
    csv: { icon: FileSpreadsheet, color: "text-green-600 dark:text-green-400" },
    xlsx: { icon: FileSpreadsheet, color: "text-green-600 dark:text-green-400" },

    // VCS
    gitignore: { icon: GitBranch, color: "text-zinc-500 dark:text-zinc-400" },
    gitattributes: { icon: GitBranch, color: "text-zinc-500 dark:text-zinc-400" },

    // Security
    pem: { icon: Shield, color: "text-red-500 dark:text-red-400" },
    key: { icon: Shield, color: "text-red-500 dark:text-red-400" },
    cert: { icon: Shield, color: "text-red-500 dark:text-red-400" },

    // Database
    db: { icon: Database, color: "text-zinc-500 dark:text-zinc-400" },
    sqlite: { icon: Database, color: "text-zinc-500 dark:text-zinc-400" },

    // Package managers
    lockfile: { icon: Lock, color: "text-zinc-400 dark:text-zinc-500" },
};

/** Filename-only matches (no extension) */
const NAME_MAP: Record<string, IconMapping> = {
    Dockerfile: { icon: Package, color: "text-blue-500 dark:text-blue-400" },
    Makefile: { icon: Terminal, color: "text-zinc-500 dark:text-zinc-400" },
    LICENSE: { icon: Shield, color: "text-zinc-500 dark:text-zinc-400" },
    CONTRIBUTING: { icon: FileText, color: "text-zinc-500 dark:text-zinc-400" },
    CHANGELOG: { icon: FileText, color: "text-zinc-500 dark:text-zinc-400" },
};

const DEFAULT: IconMapping = { icon: File, color: "text-zinc-400 dark:text-zinc-500" };

/** Get icon info for a filename */
export function getFileIcon(filename: string): IconMapping {
    // Check filename-only matches first
    const nameMatch = NAME_MAP[filename];
    if (nameMatch) return nameMatch;

    // Check special composite names
    if (filename === ".gitignore" || filename === ".gitattributes") {
        return NAME_MAP[filename.replace(".", "")] ?? EXT_MAP["gitignore"] ?? DEFAULT;
    }
    if (filename.endsWith(".lock")) {
        return EXT_MAP["lock"] ?? DEFAULT;
    }

    // Check extension
    const ext = filename.includes(".") ? filename.split(".").pop()!.toLowerCase() : "";
    return EXT_MAP[ext] ?? DEFAULT;
}
