import { Button, Row, Col, Typography, Layout } from 'antd';
import { GOOGLE_CLIENT_ID } from "../core/constants";
import useEphemeralKeyPair from "../core/useEphemeralKeyPair";
import GoogleLogo from "../components/GoogleLogo";
//import { WalletSelector } from "@aptos-labs/wallet-adapter-ant-design";
import "@aptos-labs/wallet-adapter-ant-design/dist/index.css";
import { useEffect } from "react";
import { useNavigate } from 'react-router-dom';

const { Title, Text } = Typography;

function LoginPage() {
    const navigate = useNavigate();
    const ephemeralKeyPair = useEphemeralKeyPair();

    useEffect(() => {
        const keylessAccount = localStorage.getItem('@aptos-connect/keyless-accounts');

        if (keylessAccount) {
            const accountData = JSON.parse(keylessAccount);
            const { idToken } = accountData.state.accounts[0];
            const { exp } = JSON.parse(atob(idToken.raw.split('.')[1]));


            if (exp * 1000 > Date.now()) {
                window.location.href = `/callback#id_token=${idToken.raw}`
            }
        }
    }, [navigate]);



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
        <Layout style={{ backgroundColor: '#ede8f5' }}>
            <Row align="middle" justify="center" style={{ height: '100vh', padding: '15px' }}>
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
                        icon={<GoogleLogo style={{ marginRight: 8 }} />}
                        href={redirectUrl.toString()}
                        style={{ display: 'flex', alignItems: 'center' }}
                    >
                        Sign in with Google
                    </Button>
                </Col>
            </Row>
        </Layout>
    );
}

export default LoginPage;
