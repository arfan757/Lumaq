import React from "react";

interface ArtworkProps {
  artwork: string | null;
  title?: string;
}

export const Artwork: React.FC<ArtworkProps> = ({ artwork, title = "Album Artwork" }) => {
  return (
    <div className="artwork-container">
      {artwork ? (
        <img
          src={artwork}
          alt={title}
          className="artwork-img"
          draggable={false}
        />
      ) : (
        <div className="artwork-placeholder">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <circle cx="12" cy="12" r="10" />
            <circle cx="12" cy="12" r="3" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10" />
          </svg>
        </div>
      )}
    </div>
  );
};
