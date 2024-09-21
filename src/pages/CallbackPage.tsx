import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useKeylessAccounts } from "../core/useKeylessAccounts";
import { invoke } from '@tauri-apps/api/tauri'; // For calling Rust backend
import Nav from "./Nav.tsx";
import { PeerInfo } from "../core/types.ts";

function CallbackPage() {
    const isLoading = useRef(false);
    const switchKeylessAccount = useKeylessAccounts(
        (state: { switchKeylessAccount: any; }) => state.switchKeylessAccount
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

                // Fetch peer info and redirect based on node type
                const peerInfo: PeerInfo = await invoke('load_peer');
                if (peerInfo && peerInfo.node_type) {
                    const nodeType = peerInfo.node_type;
                    if (nodeType === "Provider") {
                        navigate("/dashboard");
                    } else if (nodeType === "Distributor") {
                        navigate("/node-dashboard");
                    }
                }
            } catch (error) {
                navigate("/");
                console.log(error)
            }
        }

        if (!idToken) {
            navigate("/");
            return;
        }

        deriveAccount(idToken);
    }, [idToken, isLoading, navigate, switchKeylessAccount]);

    return (
        <div className="flex items-center justify-center h-screen w-screen">
            <Nav />
            <div className="relative flex justify-center items-center border rounded-lg px-8 py-2 shadow-sm cursor-not-allowed tracking-wider">
                <span className="absolute flex h-3 w-3 -top-1 -right-1">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
                </span>
                Redirecting...
            </div>
        </div>
    );
}

export default CallbackPage;
