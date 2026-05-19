import type { Components } from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeHighlight from 'rehype-highlight'
import rehypeKatex from 'rehype-katex'

/** Reusable Markdown components that apply dir="auto" to block elements. */
export const markdownComponents: Partial<Components> = {
    p: (props) => <p dir="auto" {...props} />,
    ul: (props) => <ul dir="auto" {...props} />,
    ol: (props) => <ol dir="auto" {...props} />,
    h1: (props) => <h1 dir="auto" {...props} />,
    h2: (props) => <h2 dir="auto" {...props} />,
    h3: (props) => <h3 dir="auto" {...props} />,
    h4: (props) => <h4 dir="auto" {...props} />,
    h5: (props) => <h5 dir="auto" {...props} />,
    h6: (props) => <h6 dir="auto" {...props} />,
    blockquote: (props) => <blockquote dir="auto" {...props} />,
}

/** Standard plugin configuration for all Markdown renderers. */
export const markdownPlugins = {
    remarkPlugins: [remarkGfm, remarkMath],
    rehypePlugins: [rehypeHighlight, rehypeKatex],
    components: markdownComponents,
}