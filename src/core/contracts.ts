
import { Aptos, AptosConfig, KeylessAccount, U64 } from "@aptos-labs/ts-sdk";
const API_KEY = import.meta.env.VITE_API_KEY;
const moduleAddress = import.meta.env.VITE_moduleAddress;

const config = new AptosConfig({
  fullnode: `https://aptos-testnet.nodit.io/${API_KEY}/v1`,
  indexer: `https://aptos-testnet.nodit.io/${API_KEY}/v1/graphql`,
});

const aptos = new Aptos(config);

export const upload_content = async (
    account: KeylessAccount,
    cid: string,
    nodes: string[],
    fee_paid: U64,
    consumer_fee: U64,
    file_type: string
  ) => {
    try {
      // Build a simple transaction using aptos SDK
      const rawTransaction = await aptos.transaction.build.simple({
        sender: account.accountAddress.bcsToHex().toString(),
        data: {
            function: `${moduleAddress}::BoxPeer::upload_content`,
            typeArguments: [],
            functionArguments: [cid, nodes, fee_paid, consumer_fee, file_type]
        }
    });

    const signedTxn = await aptos.signAndSubmitTransaction({ signer: account, transaction: rawTransaction });
      // Wait for the transaction to be confirmed
      await aptos.waitForTransaction({ transactionHash: signedTxn.hash });

      return `Success. Transaction hash ${signedTxn.hash}`;
    } catch (error: any) {
      console.error('Error submitting transaction:', error.message);
      return `Failed to upload content: ${error.message}`;
    }
  };

export const queryRegistry = async (accountAddress: string, cid: string): Promise<string> => {
  try {
    const result = await aptos.getAccountResource({
      accountAddress,
      resourceType: `${moduleAddress}::BoxPeer::ContentRegistry`,
    });


    console.log("Content Registry resources", result)
    const contentRegistry = result.contents;
    const content = contentRegistry.find((content: any) => content.cid === cid);

    if (!content) {
      console.error(`No content found with cid: ${cid}`);
      return "";
    }

    // Return the file_type for the matched cid
    return content.file_type;
  } catch (error: any) {
    console.error("Error fetching content registry:", error);
    return "";
  }
};

// Function to query and fetch all Content events globally and collect CIDs
export const fetchAllContentCIDs = async (): Promise<string[]> => {
    const queryContentEvents = `
query EventQuery {
  events(
    offset: 0
    limit: 1
    where: {
      type: {_eq: "0xbca47e0e304b5dcd2b54c9d6683d1cd11010d6453798da34acd1ae5065c4ff5f::BoxPeer::Content"}
    }
  ) {
    transaction_version
    account_address
    creation_number
    event_index
    type
    data
  }
}
  `;
    try {
        if (config.indexer) {
            const response = await fetch(config.indexer, {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  query: queryContentEvents,
                }),
              });

              const result = await response.json();

              if (result.errors) {
                console.error("GraphQL query failed", result.errors);
                return [];
              }
                    // Extract and filter CIDs from the events
      const events = result.data?.events || [];
      const cids = events.map((event: any) => event.attributes.cid);

      console.log("Fetched CIDs:", cids);

      return cids;
        }
        return [];

    } catch (error: any) {
      console.error("Error fetching content events:", error);
      return [];
    }
  };
