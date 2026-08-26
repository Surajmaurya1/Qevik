import React, { useEffect, useState } from 'react';
import { AppWindow, FileText, Folder, Terminal, Calculator, Globe } from 'lucide-react';
import { ResultType } from '../../types/results';
import { getIcon } from '../../lib/ipc';

interface IconProps {
  iconId: string | null;
  resultType: ResultType;
  className?: string;
}

export const Icon: React.FC<IconProps> = React.memo(({ iconId, resultType, className = '' }) => {
  const [iconData, setIconData] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;
    if (iconId) {
      getIcon(iconId)
        .then((data) => {
          if (isMounted && data) {
            setIconData(data);
          }
        })
        .catch(() => {
          // Fallback to vector icon
        });
    } else {
      setIconData(null);
    }

    return () => {
      isMounted = false;
    };
  }, [iconId]);

  if (iconData) {
    return (
      <img
        src={`data:image/png;base64,${iconData}`}
        alt=""
        className={`anim-icon-fade ${className}`}
        style={{ width: 24, height: 24, objectFit: 'contain' }}
      />
    );
  }

  const iconSize = 20;
  const strokeWidth = 1.75;

  switch (resultType) {
    case 'app':
      return <AppWindow size={iconSize} strokeWidth={strokeWidth} className={className} />;
    case 'folder':
      return <Folder size={iconSize} strokeWidth={strokeWidth} className={className} />;
    case 'file':
      return <FileText size={iconSize} strokeWidth={strokeWidth} className={className} />;
    case 'command':
      return <Terminal size={iconSize} strokeWidth={strokeWidth} className={className} />;
    case 'calculator':
      return <Calculator size={iconSize} strokeWidth={strokeWidth} className={className} />;
    case 'web':
      return <Globe size={iconSize} strokeWidth={strokeWidth} className={className} />;
    default:
      return <AppWindow size={iconSize} strokeWidth={strokeWidth} className={className} />;
  }
});

Icon.displayName = 'Icon';
