//use sbor::*;
use scrypto::prelude::*;

mod dex;

blueprint! {

    pub struct Market {
        //manage open orders user access
        orders_badge_minter: Vault,
        orders_badge_def: ResourceDef,

        // Define market data
        pub dex: dex::Dex,
        name: String,
    }

    impl Market {
        // create a market with specified quote and base token as initiale quantity in the vault.
        pub fn instantiate_market(
            quote_token: Address,
            base_token: Address,
            name: String,
        ) -> (Component, Bucket) {
            // Create a badge for internal use which will hold mint/burn authority for the admin badge we will soon create
            let orders_badge_minter: Bucket =
                ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", &format!("market:{}", name))
                .initial_supply_fungible(1);

            // Define the admin badge
            let admin_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", &format!("Market {} access Badge", name))
                .initial_supply_fungible(1);


            // Create the ResourceDef for a mutable supply admin badge
            let orders_badge_def = ResourceBuilder::new_non_fungible()
                .metadata("name", name.clone())
                .flags(MINTABLE )
                .badge(orders_badge_minter.resource_def(), MAY_MINT)
                .no_initial_supply();

            (
                Self {
                    orders_badge_minter: Vault::with_bucket(orders_badge_minter),
                    orders_badge_def,
                    name,
                    dex: dex::Dex::new(quote_token, base_token),
                }
                .instantiate(),
                admin_badge
            )
        }

        pub fn create_openorders(&mut self) -> Bucket {
            let badge = self.orders_badge_minter.authorize(|auth| {
                self.orders_badge_def.mint_non_fungible(
                    &NonFungibleKey::from(Uuid::generate()),
                    BadgeData {
                        name: self.name.clone(),
                    },
                    auth,
                )
            });

            info!("badge:{:?}", badge.get_non_fungible_keys().get(0).unwrap());

            let orders = dex::UserOrders::new(self.dex.quote_token.clone(), self.dex.base_token.clone());
            self.dex.user_orders.insert(
                badge.get_non_fungible_keys().get(0).unwrap().clone(),
                orders,
            );

            badge
        }

        //#[auth(orders_badge_def)]
        pub fn buy_order(&mut self, price: Decimal, amount: Decimal, quote: Bucket, auth: BucketRef) -> Vec<Bucket> {
            info!("buy_order");
            let owner_keys = auth.get_non_fungible_keys();
            let data: BadgeData = auth
                .resource_def()
                .get_non_fungible_data(owner_keys.get(0).unwrap());
            assert!(data.name == self.name, "Not current market open order badge");
            self.dex.bid(
                owner_keys.get(0).unwrap().clone(),
                price,
                amount,
                quote,
                dex::OrderType::Limit,
            ).1
        }

        //#[auth(orders_badge_def)]
        pub fn sell_order(&mut self, price: Decimal, amount: Decimal, base: Bucket, auth: BucketRef) -> Vec<Bucket> {
            info!("sell order");
            let owner_keys = auth.get_non_fungible_keys();
            let data: BadgeData = auth
                .resource_def()
                .get_non_fungible_data(owner_keys.get(0).unwrap());
            assert!(data.name == self.name, "Not current market open order badge");
            self.dex.ask(
                owner_keys.get(0).unwrap().clone(),
                price,
                amount,
                base,
                dex::OrderType::PostOnly,
            ).1
        }
    }

}

#[derive(Debug, Clone, NonFungibleData)]
pub struct BadgeData {
    name: String,
}

/*fn find_by_side<T: std::cmp::Ord>(set: &BTreeSet<T>, side: Side) -> Option<&T> {
    match side {
        Side::Bid => set.iter().max(), //max
        Side::Ask => set.iter().min(), //min
    }
}


#[derive(Debug, TypeId, Encode, Decode, Describe, NonFungibleData)]
pub struct OpenOrders {
    pub quote_vault: Vault,
    pub base_vault: Vault,
    pub orders: Vec<Order>,
}

impl OpenOrders {
    pub fn new(quote: ResourceDef, base: ResourceDef) -> Self {
        Self {
            orders: vec![],
            quote_vault: Vault::new(quote),
            base_vault: Vault::new(base),
        }
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct Order {
    pub id: u32,
    pub price: Decimal,
    pub amount: Decimal, //amount in base to trade.
    pub provided_amount: Decimal,
    pub order_type: OrderType,
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    ImmediateOrCancel,
    PostOnly,
}*/