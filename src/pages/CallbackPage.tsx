import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useKeylessAccounts } from "../core/useKeylessAccounts";
import { Spin } from "antd";

function CallbackPage() {
    const isLoading = useRef(false);
    const switchKeylessAccount = useKeylessAccounts(
        (state: { switchKeylessAccount: any }) => state.switchKeylessAccount
    );
    const navigate = useNavigate();

    const fragmentParams = new URLSearchParams(window.location.hash.substring(1));
    const idToken = fragmentParams.get("id_token");

    useEffect(() => {
        if (isLoading.current) return;
        isLoading.current = true;

        async function deriveAccount(idToken: string) {
            try {
                await switchKeylessAccount(idToken);
                navigate("/dashboard");
            } catch (error) {
                console.error(error);
                navigate("/");
            }
        }

        if (!idToken) {
            navigate("/");
            return;
        }

        deriveAccount(idToken);
    }, [idToken, navigate, switchKeylessAccount]);

    return (
        <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh', width: '100vw' }}>
            <Spin tip="Redirecting..." size="large" />
        </div>
    );
}

export default CallbackPage;
