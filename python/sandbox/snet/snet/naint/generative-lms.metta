!(import! &self snet_io)

!(add-reduct &self
  (= (generative-lms)
     (snet-service "naint" "generative-lms"
       ; Put your data here or use environment variables
       ;(Kwargs
       ;  (private_key SNET_PRIVATE_KEY)
       ;  (eth_rpc_endpoint ETH_RPC_ENDPOINT)
       ;  (email SNET_EMAIL)
       ;  (free_call_auth_token_bin FREE_CALL_AUTH_TOKEN_BIN)
       ;)
     )))
