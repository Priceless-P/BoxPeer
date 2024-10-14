import { useState } from "react";
import { fetchAllContentCIDs, payForContent, getPurchasersByCid } from "../core/contracts";
import { FileObject } from "../pages/types";
import { useKeylessAccounts } from "../core/useKeylessAccounts";
import { message, Card, Button, Tooltip, Spin } from "antd";
import { EyeOutlined, DollarOutlined } from '@ant-design/icons';
import { usePreview } from '../context/PreviewContext';
import { KeylessAccount } from "@aptos-labs/ts-sdk";

export const useFileManager = () => {
  const [fileObjects, setFileObjects] = useState<FileObject[]>([]);
  const [loading, setLoading] = useState(false);
  const { activeAccount } = useKeylessAccounts();
  const { updatePreviewContent } = usePreview();

  const getMimeType = (fileType: string | undefined) => {
    switch (fileType) {
      case 'jpg': return 'image/jpeg';
      case 'png': return 'image/png';
      case 'txt': return 'text/plain';
      case 'pdf': return 'application/pdf';
      case 'mp3': return 'audio/mpeg';
      case 'mp4': return 'video/mp4';
      case 'mkv': return 'video/x-matroska';
      default: return 'application/octet-stream';
    }
  };


  const hasPurchasedContent = async (cid: string, account: KeylessAccount) => {
    try {
      const purchasers = await getPurchasersByCid(cid);

      const result = purchasers.flat().includes(account.accountAddress.toString());

      console.log(result)
      return result
    } catch (error) {
      console.error("Failed to fetch purchasers:", error);
      return false;
    }
  };

  const handlePay = async (cid: string) => {
    try {
      if (activeAccount) {
        setLoading(true);
        const hide = message.loading("Processing payment...", 0);
        const result = await payForContent(activeAccount, cid);
        hide();
        message.success(result);
      } else {
        message.error("Connect wallet to proceed");
      }
    } catch (error) {
      message.error("An error occurred while processing payment");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const getPreview = async (fileObject: FileObject, fileURL: string) => {
    const { title, description, fileType, consumerFee, cid } = fileObject;

    let filePreview;
    if (fileType === 'jpg' || fileType === 'png') {
      filePreview = <img src={fileURL} alt={title} style={{ widows: '80%', borderRadius: '8px' }} />;
    } else if (fileType === 'mp4' || fileType === 'mkv') {
      filePreview = <video controls style={{ width: '80%' }} src={fileURL} />;
    } else if (fileType === 'mp3') {
      filePreview = <audio controls src={fileURL} />;
    } else if (fileType === 'pdf') {
      filePreview = <iframe src={fileURL} style={{ width: '80%',  borderRadius: '8px' }} />;
    } else {
      filePreview = <a href={fileURL} download>Download File</a>;
    }
    if (!activeAccount) {
        return 'You need to connect wallet to proceed'
        }
    const purchased = await hasPurchasedContent(cid, activeAccount);

    const previewElement = (
      <Card
        key={cid}
        title={<Tooltip title={description}>{title}</Tooltip>}
        extra={
          parseInt(consumerFee) ? (
            <Tooltip title={`APT ${parseInt(consumerFee) / 10 ** 8}`}>
              <DollarOutlined style={{ color: 'red' }} />
            </Tooltip>
          ) : (
            <Tooltip title="Free">
              <EyeOutlined style={{ color: 'green' }} />
            </Tooltip>
          )
        }
        hoverable
        style={{ width: '100%', marginBottom: '20px' }}
      >
        {parseInt(consumerFee) > 0 && !purchased ? (
          <div style={{ position: 'relative', opacity: 0.4 }}>
            {filePreview}
            <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)' }}>
              <Button type="primary" onClick={() => handlePay(cid)} disabled={loading}>
                {loading ? <Spin /> : "Pay to View"}
              </Button>
            </div>
          </div>
        ) : (
          filePreview
        )}
      </Card>
    );

    // Update context with the preview element and fileObject
    updatePreviewContent(cid, previewElement, fileObject);
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).then(() => {
      message.success("Copied to clipboard!");
    }).catch(err => {
      message.error("Failed to copy: " + err);
    });
  };

  const fetchFileMetadata = async () => {
    try {
      const all = await fetchAllContentCIDs();
      const fileObjects: FileObject[] = all.map((item: any) => ({
        cid: item.cid,
        owner: item.owner,
        fileType: item.file_type,
        feePaid: item.fee_paid,
        consumerFee: item.consumer_fee,
        owner_name: item.owner_name,
        title: item.title,
        description: item.description,
      }));
      setFileObjects(fileObjects);
    } catch (error) {
      console.error("Failed to fetch file metadata:", error);
    }
  };

  const fetchFileMetadataByCid = async (cid: string): Promise<FileObject | null> => {
    try {
      const all = await fetchAllContentCIDs();
      const fileObjects: FileObject[] = all.map((item: any) => ({
        cid: item.cid,
        owner: item.owner,
        fileType: item.file_type,
        feePaid: item.fee_paid,
        consumerFee: item.consumer_fee,
        owner_name: item.owner_name,
        title: item.title,
        description: item.description,
      }));

      // Find the file that matches the given CID
      const fileObject = fileObjects.find(item => item.cid === cid);
      return fileObject || null;
    } catch (error) {
      console.error("Failed to fetch file metadata:", error);
      return null;
    }
  };

  const getAllFiles = async (files: Uint8Array[]) => {
    files.forEach((fileData, index) => {
      const fileObject = fileObjects[index];
      if (!fileObject) return;

      const mimeType = getMimeType(fileObject.fileType);
      const blob = new Blob([new Uint8Array(fileData)], { type: mimeType });
      const fileURL = URL.createObjectURL(blob);

      getPreview(fileObject, fileURL);
    });
  };

  return {
    fileObjects,
    fetchFileMetadata,
    getPreview,
    fetchFileMetadataByCid,
    copyToClipboard,
    getAllFiles
  };
};
