import ExternalLinkIcon from "../data/ext-lnk.svg?react";

export default function SocialLinkButton({
    url,
    children,
}: {
    url: string;
    children: string;
}) {
    return (
        <div className="group/link-btn relative">
            <div className="absolute inset-0 link-btn-shadow" />
            <a
                href={url}
                target="_blank"
                rel="noopener noreferrer"
                className="link-btn"
            >
                {children}
                <ExternalLinkIcon className="w-3.5 h-3.5" />
            </a>
        </div>
    )
}