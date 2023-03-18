use ethers::prelude::abigen;

abigen!(
    ERC20,
    r#"[
        function transfer(address to, uint256 amount) external;
    ]"#;
    TokenDistributor,
    r#"[
        function claimableTokens(address _owner) public view returns (uint256 claimable)
        function claim() public;
    ]"#;
);
