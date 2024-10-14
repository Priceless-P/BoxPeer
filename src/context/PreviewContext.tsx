import React, { createContext, useContext, useState } from 'react';
import { FileObject, PreviewContent, PreviewContextType } from '../pages/types';

const PreviewContext = createContext<PreviewContextType | null>(null);

export const usePreview = () => {
  const context = useContext(PreviewContext);
  if (!context) {
    throw new Error("usePreview must be used within a PreviewProvider");
  }
  return context;
};

export const PreviewProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [previewContent, setPreviewContent] = useState<PreviewContent[]>([]);

  const updatePreviewContent = (cid: string, element: JSX.Element, fileObject: FileObject) => {
    setPreviewContent((prev) => {
      const existingIndex = prev.findIndex((item) => item.cid === cid);
      if (existingIndex > -1) {
        const updatedContent = [...prev];
        updatedContent[existingIndex] = { cid, element, fileObject };
        return updatedContent;
      } else {
        return [...prev, { cid, element, fileObject }];
      }
    });
  };

  const getPreviewByCid = (cid: string) => {
    return previewContent.find((item) => item.cid === cid);
  };

  return (
    <PreviewContext.Provider value={{ previewContent, updatePreviewContent, getPreviewByCid }}>
      {children}
    </PreviewContext.Provider>
  );
};
