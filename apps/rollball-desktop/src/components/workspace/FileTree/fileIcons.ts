import type { ComponentType } from "react";
import {
    DiRust,
    DiJavascript,
    DiHtml5,
    DiCss3Full,
    DiSass,
    DiPython,
    DiGo,
    DiDocker,
    DiGit,
    DiNodejsSmall,
    DiReact,
    DiJava,
    DiSwift,
    DiRuby,
    DiPhp,
    DiMongodb,
    DiMarkdown,
} from "react-icons/di";
import {
    SiTypescript,
    SiTailwindcss,
    SiWebpack,
    SiVite,
    SiEslint,
    SiPrettier,
    SiVuedotjs,
} from "react-icons/si";
import {
    File,
    FileText,
    FileJson,
    FileType,
    FileImage,
    FileVideo,
    FileAudio,
    FileArchive,
    FileSpreadsheet,
    Package,
    Settings,
    Shield,
    Database,
    Terminal,
    Lock,
} from "lucide-react";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type IconComponent = ComponentType<any>;

interface IconMapping {
    icon: IconComponent;
    color: string; // CSS color or Tailwind class
    isDevicon?: boolean; // true = icon has built-in brand color, ignore `color` class
}

/** Map file extension to icon and color */
const EXT_MAP: Record<string, IconMapping> = {
    // Rust
    rs: { icon: DiRust, color: "", isDevicon: true },
    toml: { icon: Settings, color: "#9CA3AF" },
    lock: { icon: Lock, color: "#9CA3AF" },

    // TypeScript / JavaScript
    ts: { icon: SiTypescript, color: "", isDevicon: true },
    tsx: { icon: DiReact, color: "", isDevicon: true },
    js: { icon: DiJavascript, color: "", isDevicon: true },
    jsx: { icon: DiReact, color: "", isDevicon: true },
    mjs: { icon: DiJavascript, color: "", isDevicon: true },
    cjs: { icon: DiJavascript, color: "", isDevicon: true },

    // Web
    html: { icon: DiHtml5, color: "", isDevicon: true },
    css: { icon: DiCss3Full, color: "", isDevicon: true },
    scss: { icon: DiSass, color: "", isDevicon: true },
    less: { icon: DiCss3Full, color: "", isDevicon: true },
    svg: { icon: FileImage, color: "#F59E0B" },

    // Data / Config
    json: { icon: FileJson, color: "#EAB308" },
    yaml: { icon: FileJson, color: "#EF4444" },
    yml: { icon: FileJson, color: "#EF4444" },
    xml: { icon: FileText, color: "#F97316" },

    // Markdown / Docs
    md: { icon: DiMarkdown, color: "#1565C0", isDevicon: false },
    mdx: { icon: DiMarkdown, color: "#1565C0", isDevicon: false },
    txt: { icon: FileText, color: "#9CA3AF" },
    pdf: { icon: FileType, color: "#EF4444" },

    // Shell
    sh: { icon: Terminal, color: "#22C55E" },
    bash: { icon: Terminal, color: "#22C55E" },
    zsh: { icon: Terminal, color: "#22C55E" },
    fish: { icon: Terminal, color: "#22C55E" },
    ps1: { icon: Terminal, color: "#3B82F6" },
    bat: { icon: Terminal, color: "#6B7280" },
    cmd: { icon: Terminal, color: "#6B7280" },

    // Python
    py: { icon: DiPython, color: "", isDevicon: true },
    pyi: { icon: DiPython, color: "", isDevicon: true },

    // Go
    go: { icon: DiGo, color: "", isDevicon: true },
    mod: { icon: Package, color: "#06B6D4" },
    sum: { icon: Lock, color: "#9CA3AF" },

    // Java / JVM
    java: { icon: DiJava, color: "", isDevicon: true },
    kt: { icon: FileText, color: "#7C3AED" },
    scala: { icon: FileText, color: "#EF4444" },

    // Swift
    swift: { icon: DiSwift, color: "", isDevicon: true },

    // Ruby
    rb: { icon: DiRuby, color: "", isDevicon: true },

    // PHP
    php: { icon: DiPhp, color: "", isDevicon: true },

    // Images
    png: { icon: FileImage, color: "#A855F7" },
    jpg: { icon: FileImage, color: "#A855F7" },
    jpeg: { icon: FileImage, color: "#A855F7" },
    gif: { icon: FileImage, color: "#A855F7" },
    webp: { icon: FileImage, color: "#A855F7" },
    ico: { icon: FileImage, color: "#A855F7" },
    bmp: { icon: FileImage, color: "#A855F7" },

    // Media
    mp4: { icon: FileVideo, color: "#EC4899" },
    mov: { icon: FileVideo, color: "#EC4899" },
    avi: { icon: FileVideo, color: "#EC4899" },
    webm: { icon: FileVideo, color: "#EC4899" },
    mp3: { icon: FileAudio, color: "#EC4899" },
    wav: { icon: FileAudio, color: "#EC4899" },
    flac: { icon: FileAudio, color: "#EC4899" },

    // Archives
    zip: { icon: FileArchive, color: "#EAB308" },
    tar: { icon: FileArchive, color: "#EAB308" },
    gz: { icon: FileArchive, color: "#EAB308" },
    rar: { icon: FileArchive, color: "#EAB308" },
    "7z": { icon: FileArchive, color: "#EAB308" },

    // Spreadsheets
    csv: { icon: FileSpreadsheet, color: "#22C55E" },
    xlsx: { icon: FileSpreadsheet, color: "#22C55E" },
    xls: { icon: FileSpreadsheet, color: "#22C55E" },

    // Database
    db: { icon: Database, color: "#6B7280" },
    sqlite: { icon: Database, color: "#6B7280" },
    sql: { icon: DiMongodb, color: "#22C55E" },

    // Security
    pem: { icon: Shield, color: "#EF4444" },
    key: { icon: Shield, color: "#EF4444" },
    cert: { icon: Shield, color: "#EF4444" },
    crt: { icon: Shield, color: "#EF4444" },

    // C/C++
    c: { icon: FileText, color: "#3B82F6" },
    h: { icon: FileText, color: "#3B82F6" },
    cpp: { icon: FileText, color: "#6366F1" },
    hpp: { icon: FileText, color: "#6366F1" },

    // C#
    cs: { icon: FileText, color: "#22C55E" },

    // VCS
    gitignore: { icon: DiGit, color: "", isDevicon: true },
    gitattributes: { icon: DiGit, color: "", isDevicon: true },

    // Package managers
    lockfile: { icon: Lock, color: "#9CA3AF" },
};

/** Filename-only matches (no extension) */
const NAME_MAP: Record<string, IconMapping> = {
    Dockerfile: { icon: DiDocker, color: "", isDevicon: true },
    Makefile: { icon: Terminal, color: "#6B7280" },
    LICENSE: { icon: Shield, color: "#6B7280" },
    CONTRIBUTING: { icon: FileText, color: "#6B7280" },
    CHANGELOG: { icon: FileText, color: "#6B7280" },
    "package.json": { icon: DiNodejsSmall, color: "", isDevicon: true },
    "tsconfig.json": { icon: SiTypescript, color: "", isDevicon: true },
    ".eslintrc": { icon: SiEslint, color: "", isDevicon: true },
    ".eslintrc.js": { icon: SiEslint, color: "", isDevicon: true },
    ".eslintrc.json": { icon: SiEslint, color: "", isDevicon: true },
    ".prettierrc": { icon: SiPrettier, color: "", isDevicon: true },
    "tailwind.config.js": { icon: SiTailwindcss, color: "", isDevicon: true },
    "tailwind.config.ts": { icon: SiTailwindcss, color: "", isDevicon: true },
    "vite.config.ts": { icon: SiVite, color: "", isDevicon: true },
    "vite.config.js": { icon: SiVite, color: "", isDevicon: true },
    "webpack.config.js": { icon: SiWebpack, color: "", isDevicon: true },
    "vue.config.js": { icon: SiVuedotjs, color: "", isDevicon: true },
};

const DEFAULT: IconMapping = { icon: File, color: "#9CA3AF" };

/** Get icon info for a filename */
export function getFileIcon(filename: string): IconMapping {
    // Check filename-only matches first (exact filename like "package.json")
    const nameMatch = NAME_MAP[filename];
    if (nameMatch) return nameMatch;

    // Check special dot-files
    if (filename === ".gitignore" || filename === ".gitattributes") {
        return EXT_MAP["gitignore"] ?? DEFAULT;
    }
    if (filename.endsWith(".lock")) {
        return EXT_MAP["lock"] ?? DEFAULT;
    }

    // Check extension
    const ext = filename.includes(".") ? filename.split(".").pop()!.toLowerCase() : "";
    return EXT_MAP[ext] ?? DEFAULT;
}
