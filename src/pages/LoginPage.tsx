import {Button, Row, Col, Typography, Layout, Radio, Modal,} from 'antd';
import { GOOGLE_CLIENT_ID } from "../core/constants";
import useEphemeralKeyPair from "../core/useEphemeralKeyPair";
import GoogleLogo from "../components/GoogleLogo";
//import { WalletSelector } from "@aptos-labs/wallet-adapter-ant-design";
import "@aptos-labs/wallet-adapter-ant-design/dist/index.css";
import Nav from "./Nav.tsx";
import { PeerInfo } from '../core/types.ts';
import {invoke} from '@tauri-apps/api/tauri';
import {useEffect, useState} from "react";

const { Title, Text } = Typography;

function LoginPage() {
  const [peerType, setPeerType] = useState<string | null>(null); // Store selected peer type
  const [showModal, setShowModal] = useState(false);

  const ephemeralKeyPair = useEphemeralKeyPair();
useEffect(()=> {
  async function checkPeerType() {
    const peerInfo: PeerInfo = await invoke('load_peer');
    if (!peerInfo || !peerInfo.node_type) {
      setShowModal(true);
    } else {
      setPeerType(peerInfo.node_type)
      if (peerType) {
        localStorage.setItem("node_type", peerType);
      }
    }
  } checkPeerType();
}, [])

  const handleSelectPeerType = async (type: string) => {
    setPeerType(type);
      try {
          await invoke('save_peer', { nodeType: type });
      } catch(error: any) {
        console.log(error)
        }
    setShowModal(false);
  };

  const redirectUrl = new URL("https://accounts.google.com/o/oauth2/v2/auth");

  const searchParams = new URLSearchParams({
    client_id: GOOGLE_CLIENT_ID,
    redirect_uri: `${window.location.origin}/callback`,
    response_type: "id_token",
    scope: "openid email profile",
    nonce: ephemeralKeyPair.nonce,
  });
  redirectUrl.search = searchParams.toString();

  return (
      <Layout style={ {backgroundColor: '#ede8f5'}}>
        <Nav />
      <Row align="middle" justify="center" style={{ height: '100vh', padding: '15px'}}>
        <Col style={{ textAlign: 'center' }}>
          <Title level={1} style={{ marginBottom: '16px' }}>Welcome to BoxPeer</Title>
          <Col span={12} style={{ textAlign: "right", paddingRight: "200px" }}>
            {/*<WalletSelector />*/}
          </Col>

          <Text style={{ fontSize: '18px', marginBottom: '24px', display: 'block' }}>
            Sign in with your Google account to continue
          </Text>
          <Button
              type="primary"
              size="large"
              icon={<GoogleLogo style={{ marginRight: 8 }} />} // GoogleLogo component as an icon
              href={redirectUrl.toString()}
              style={{ display: 'flex', alignItems: 'center' }}
          >
            Sign in with Google
          </Button>
        </Col>
      </Row>

        {/* Modal to select peer type */}
        <Modal
            title="Select Your Peer Type"
            open={showModal}
            onCancel={() => setShowModal(false)}
            footer={[
              <Button key="close" onClick={() => setShowModal(false)}>
                Close
              </Button>
            ]}
        >
          <Radio.Group onChange={(e) => handleSelectPeerType(e.target.value)}>
            <Radio.Button value="Provider">Provider</Radio.Button>
            <Radio.Button value="Distributor">Distributor</Radio.Button>
            {/*<Radio.Button value="Consumer">Consumer</Radio.Button>*/}
          </Radio.Group>
        </Modal>

      </Layout>
  );
}

export default LoginPage;
