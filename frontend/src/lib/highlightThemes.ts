import githubLight from 'highlight.js/styles/github.css?inline'
import githubDark from 'highlight.js/styles/github-dark.css?inline'
import monokai from 'highlight.js/styles/monokai.css?inline'
import atomOneLight from 'highlight.js/styles/atom-one-light.css?inline'
import atomOneDark from 'highlight.js/styles/atom-one-dark.css?inline'

export const themeCSS: Record<string, string> = {
    'github': githubLight,
    'github-dark': githubDark,
    'monokai': monokai,
    'atom-one-light': atomOneLight,
    'atom-one-dark': atomOneDark,
}

export const themeNames = Object.keys(themeCSS)

/** Return the effective CSS theme name based on user selection and dark mode */
export function getEffectiveTheme(
    codeTheme: string | undefined,
    dark: boolean
): string {
    if (!codeTheme || codeTheme === 'auto') {
        return dark ? 'github-dark' : 'github'
    }
    // Fallback to github-dark if the chosen theme is missing
    return themeCSS[codeTheme] ? codeTheme : 'github-dark'
}