export interface PeerDashboardProps {}

export interface Peer {
    id: string;
}

export interface ProvidedFile {
    name: string;
    size: number;
    type: string;
}
export interface FileObject {
    cid: string;
    owner: string;
    fileType: string;
    feePaid: string;
    consumerFee: string;
      owner_name: string;
      title: string;
      description: string;
  }

  export type PreviewContent = {
    cid: string;
    element: JSX.Element;
    fileObject: FileObject;
  };

export interface PreviewContextType {
    previewContent: PreviewContent[];
    updatePreviewContent: (cid: string, element: JSX.Element, fileObject: FileObject) => void;
    getPreviewByCid: (cid: string) => PreviewContent | undefined;
  }