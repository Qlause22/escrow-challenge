#!/bin/bash

set -e

resim reset

echo -e "\n1. Create Account 1."
output=$(resim new-account)

account_address_1=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_1=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_1=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_1=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_1=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 1 : "$account_address_1

echo -e "\n2. Publish Escrow blueprint."
output=$(resim publish .)
package_address=$(echo "$output" | grep "New Package" | grep -o "package_.*")
resim show $package_address
export escrow_package=$package_address

echo -e "\n3. Create owner badge."
owner=$(resim new-simple-badge | grep -o "resource.*"$)
echo $owner

echo -e "\n4. Instansiate Escrow package."
output=$(account_1=${account_address_1} \
  escrow_package_address=${escrow_package} \
  owner=${owner} \
  resim run manifest/basic_swap/01_instantiate.rtm)
escrow_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
escrow_badge_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | sed -n '1p')
ticket=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | sed -n '2p')
resim show $escrow_component_1
resim show $escrow_badge_1

echo -e "\n5. Create Assets to trade."
output=$(account_1=${account_address_1} \
  resim run manifest/basic_swap/02_create_nft.rtm)
nft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to offer : "$nft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/basic_swap/02_create_nft.rtm)
nft_resource_address_2=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to require : "$nft_resource_address_2

output=$(account_1=${account_address_1} \
  resim run manifest/basic_swap/03_create_ft.rtm)
ft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to offer : "$ft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/basic_swap/03_create_ft.rtm)
ft_resource_address_2=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to require : "$ft_resource_address_2

echo -e "\n6. Create Account 2."
output=$(resim new-account)

account_address_2=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_2=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_2=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 2 : "$account_address_2

echo -e "\n7. Transfering to required assets to account 2."
output=$(account_1=${account_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_2=${nft_resource_address_2} \
  account_2=${account_address_2} \
  resim run manifest/basic_swap/transfer_assets.rtm)
echo "Account 2 state :"
resim show $account_address_2

echo -e "\n8. Register."
output=$(account_1=${account_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/basic_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

echo -e "\n9. Create Swap Contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  ft_resource_1=${ft_resource_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_1=${nft_resource_address_1} \
  nft_resource_2=${nft_resource_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/basic_swap/05_create_swap.rtm )
echo "$output" | grep -o "Transaction.*" 
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n10. Cancel contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  escrow_component=${escrow_component_1} \
  swap_contract=${swap_component_1} \
  resim run manifest/basic_swap/06_cancel_swap.rtm)
echo "$output" | grep -o "Transaction.*" 
echo $swap_component_1 "Cancelled."
resim show

echo -e "\11. Create Swap Contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  ft_resource_1=${ft_resource_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_1=${nft_resource_address_1} \
  nft_resource_2=${nft_resource_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/basic_swap/05_create_swap.rtm )
echo "$output" | grep -o "Transaction.*" 
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n12. Withdrawing assets (error intended)."
output=$(account_1=${account_address_1} \
  ticket=${ticket} \
  escrow_component=${escrow_component_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  swap_contract=${swap_component_1} \
  resim run manifest/basic_swap/07_withdraw_assets.rtm || true)

echo -e "\n13. Set default address to account 2."
resim set-default-account $account_address_2 $private_key_2 $owner_badge_2":"$owner_badge_id_2

echo -e "\n14. Register account 2."
output=$(account_1=${account_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/basic_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

echo -e "\n15. Exchange assets."
output=$(account_2=${account_address_2} \
  badge=${escrow_badge_1} \
  escrow_component=${escrow_component_1} \
  id="#2#" \
  nft_resource_2=${nft_resource_address_2} \
  ft_resource_2=${ft_resource_address_2} \
  swap_contract=${swap_component_1} \
  ticket=${ticket} \
  resim run manifest/basic_swap/08_exchange_assets.rtm)

echo "$output" 

resim show $account_address_2

echo -e "\n16. Set default account to address 1 and withdraw all assets."
resim set-default-account $account_address_1 $private_key_1 $owner_badge_1":"$owner_badge_id_1

echo -e "\n17. Withdrawing Assets."

output=$(account_1=${account_address_1} \
  ticket=${ticket} \
  escrow_component=${escrow_component_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  swap_contract=${swap_component_1} \
  resim run manifest/basic_swap/07_withdraw_assets.rtm | grep -o "Transaction.*")

echo $output 

resim show $account_address_1


echo -e "\n create bidable_swap."


resim reset

echo -e "\n1. Create Account 1."
output=$(resim new-account)

account_address_1=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_1=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_1=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_1=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_1=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 1 : "$account_address_1

echo -e "\n2. Publish Escrow blueprint."
output=$(resim publish .)
package_address=$(echo "$output" | grep "New Package" | grep -o "package_.*")
resim show $package_address
export escrow_package=$package_address

echo -e "\n3. Create owner badge."
owner=$(resim new-simple-badge | grep -o "resource.*"$)
echo $owner

echo -e "\n4. Instansiate Escrow package."
output=$(account_1=${account_address_1} \
  escrow_package_address=${escrow_package} \
  owner=${owner} \
  resim run manifest/bidable_swap/01_instantiate.rtm)
escrow_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
escrow_badge_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | sed -n '1p')
ticket=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | sed -n '2p')
resim show $escrow_component_1
resim show $escrow_badge_1

echo -e "\n5. Create Assets to trade."
output=$(account_1=${account_address_1} \
  resim run manifest/bidable_swap/02_create_nft.rtm)
nft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to offer : "$nft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/bidable_swap/03_create_ft.rtm)
ft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to offer : "$ft_resource_address_1

echo -e "\n6. Create Account 2."
output=$(resim new-account)

account_address_2=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_2=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_2=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 2 : "$account_address_2

echo -e "\n7. Create Account 3."
output=$(resim new-account)

account_address_3=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_3=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_3=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_3=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_3=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 2 : "$account_address_3

echo -e "\n8. Create Account 4."
output=$(resim new-account)

account_address_4=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_4=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_4=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_4=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_4=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 2 : "$account_address_4


echo -e "\n9. Register All Account."
output=$(account_1=${account_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/bidable_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

resim set-default-account $account_address_2 $private_key_2 $owner_badge_2":"$owner_badge_id_2
output=$(account_1=${account_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/bidable_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

resim set-default-account $account_address_3 $private_key_3 $owner_badge_3":"$owner_badge_id_3
output=$(account_1=${account_address_3} \
  escrow_component=${escrow_component_1} \
  resim run manifest/bidable_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

resim set-default-account $account_address_4 $private_key_4 $owner_badge_4":"$owner_badge_id_4
output=$(account_1=${account_address_4} \
  escrow_component=${escrow_component_1} \
  resim run manifest/bidable_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

resim set-default-account $account_address_1 $private_key_1 $owner_badge_1":"$owner_badge_id_1

echo -e "\n10. Create Bid-able Swap Contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  ft_resource_1=${ft_resource_address_1} \
  nft_resource_1=${nft_resource_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/bidable_swap/05_create_bidable_swap.rtm )
echo "$output" | grep -o "Transaction.*" 
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n11. Cancel contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  escrow_component=${escrow_component_1} \
  swap_contract=${swap_component_1} \
  resim run manifest/bidable_swap/06_cancel_bidable_swap.rtm)
echo "$output" | grep -o "Transaction.*" 
echo $swap_component_1 "Cancelled."
resim show

echo -e "\n12. re-Create Bid-able Swap Contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  ft_resource_1=${ft_resource_address_1} \
  nft_resource_1=${nft_resource_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/bidable_swap/05_create_bidable_swap.rtm )
echo "$output" | grep -o "Transaction.*" 
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n13. Withdrawing assets (error intended)."
output=$(account_1=${account_address_1} \
  ticket=${ticket} \
  escrow_component=${escrow_component_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  swap_contract=${swap_component_1} \
  resim run manifest/bidable_swap/07_withdraw_assets.rtm || true)

echo -e "\n14. Set default address to account 2."
resim set-default-account $account_address_2 $private_key_2 $owner_badge_2":"$owner_badge_id_2

echo -e "\n15. bid contract with account 2."
output=$(account_1=${account_address_2} \
  badge=${escrow_badge_1} \
  escrow_component=${escrow_component_1} \
  id="#2#" \
  swap_contract=${swap_component_1} \
  amount="10" \
  ticket=${ticket} \
  resim run manifest/bidable_swap/09_bid_contract.rtm | grep -o "Transaction.*")

echo "$output" 

echo -e "\n16. Set default address to account 3."
resim set-default-account $account_address_3 $private_key_3 $owner_badge_3":"$owner_badge_id_3

echo -e "\n17. bid contract with account 3 with less than previous bid."
output=$(account_1=${account_address_3} \
  badge=${escrow_badge_1} \
  escrow_component=${escrow_component_1} \
  id="#3#" \
  amount="5" \
  swap_contract=${swap_component_1} \
  ticket=${ticket} \
  resim run manifest/bidable_swap/09_bid_contract.rtm | grep -o "Transaction.*" || true)

echo "$output" 


echo -e "\n18. Set default address to account 4."
resim set-default-account $account_address_4 $private_key_4 $owner_badge_4":"$owner_badge_id_4

echo -e "\n19. bid contract with account 4."
output=$(account_1=${account_address_4} \
  badge=${escrow_badge_1} \
  escrow_component=${escrow_component_1} \
  id="#4#" \
  amount="30" \
  swap_contract=${swap_component_1} \
  ticket=${ticket} \
  resim run manifest/bidable_swap/09_bid_contract.rtm | grep -o "Transaction.*")

echo "$output" 

echo -e "\n20. Set default address to account 1."
resim set-default-account $account_address_1 $private_key_1 $owner_badge_1":"$owner_badge_id_1


echo -e "\n21. Exchange assets by owner with current bid."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  escrow_component=${escrow_component_1} \
  id="#1#" \
  nft_resource_2=${nft_resource_address_2} \
  ft_resource_2=${ft_resource_address_2} \
  swap_contract=${swap_component_1} \
  ticket=${ticket} \
  resim run manifest/bidable_swap/08_exchange_assets.rtm)

echo "$output" 

resim show $account_address_1

echo -e "\n22. Set default account to address 4 and withdraw all assets."
resim set-default-account $account_address_4 $private_key_4 $owner_badge_4":"$owner_badge_id_4

echo -e "\n23. Withdrawing Assets."

output=$(account_1=${account_address_4} \
  ticket=${ticket} \
  escrow_component=${escrow_component_1} \
  badge=${escrow_badge_1} \
  id="#4#" \
  swap_contract=${swap_component_1} \
  resim run manifest/bidable_swap/07_withdraw_assets.rtm | grep -o "Transaction.*")

echo $output 

resim show $account_address_4



echo -e "\nCreate Offerable swap."


resim reset

echo -e "\n1. Create Account 1."
output=$(resim new-account)

account_address_1=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_1=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_1=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_1=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_1=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 1 : "$account_address_1

echo -e "\n2. Publish Escrow blueprint."
output=$(resim publish .)
package_address=$(echo "$output" | grep "New Package" | grep -o "package_.*")
resim show $package_address
export escrow_package=$package_address

echo -e "\n3. Create owner badge."
owner=$(resim new-simple-badge | grep -o "resource.*"$)
echo $owner

echo -e "\n4. Instansiate Escrow package."
output=$(account_1=${account_address_1} \
  escrow_package_address=${escrow_package} \
  owner=${owner} \
  resim run manifest/offerable_swap/01_instantiate.rtm)
escrow_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
escrow_badge_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | sed -n '1p')
ticket=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | sed -n '2p')
resim show $escrow_component_1
resim show $escrow_badge_1

echo -e "\n5. Create Assets to trade."
output=$(account_1=${account_address_1} \
  resim run manifest/offerable_swap/02_create_nft.rtm)
nft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to offer : "$nft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/offerable_swap/02_create_nft.rtm)
nft_resource_address_2=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to require : "$nft_resource_address_2

output=$(account_1=${account_address_1} \
  resim run manifest/offerable_swap/03_create_ft.rtm)
ft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to offer : "$ft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/offerable_swap/03_create_ft.rtm)
ft_resource_address_2=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to require : "$ft_resource_address_2

echo -e "\n6. Create Account 2."
output=$(resim new-account)

account_address_2=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_2=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_2=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 2 : "$account_address_2

echo -e "\n7. Transfering assets that will be offer to contract to account 2."
output=$(account_1=${account_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_2=${nft_resource_address_2} \
  account_2=${account_address_2} \
  resim run manifest/offerable_swap/transfer_assets.rtm)
echo "Account 2 state :"
resim show $account_address_2

echo -e "\n8. Register."
output=$(account_1=${account_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/offerable_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

echo -e "\n9. Create OfferAble-Swap Contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  ft_resource_1=${ft_resource_address_1} \
  nft_resource_1=${nft_resource_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/offerable_swap/05_create_offerable_swap.rtm )
echo "$output" | grep -o "Transaction.*" 
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n10. Cancel contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  escrow_component=${escrow_component_1} \
  swap_contract=${swap_component_1} \
  resim run manifest/offerable_swap/06_cancel_offerable_swap.rtm)
echo "$output" | grep -o "Transaction.*" 
echo $swap_component_1 "Cancelled."
resim show

echo -e "\11. re-Create OfferAble-Swap Contract."
output=$(account_1=${account_address_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  ft_resource_1=${ft_resource_address_1} \
  nft_resource_1=${nft_resource_address_1} \
  escrow_component=${escrow_component_1} \
  resim run manifest/offerable_swap/05_create_offerable_swap.rtm )
echo "$output" | grep -o "Transaction.*" 
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n12. Set default address to account 2."
resim set-default-account $account_address_2 $private_key_2 $owner_badge_2":"$owner_badge_id_2

echo -e "\n13. Register account 2."
output=$(account_1=${account_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/offerable_swap/04_register.rtm | grep -o "Transaction.*")
echo $output

echo -e "\n14. offer contract with account 2's assets."
output=$(account_2=${account_address_2} \
  badge=${escrow_badge_1} \
  escrow_component=${escrow_component_1} \
  id="#2#" \
  nft_resource_2=${nft_resource_address_2} \
  ft_resource_2=${ft_resource_address_2} \
  swap_contract=${swap_component_1} \
  ticket=${ticket} \
  resim run manifest/offerable_swap/09_offer_contract.rtm)

echo "$output" 

resim show $account_address_2

echo -e "\n15. Set default account to address 1 and agree with account 2's offer and withdraw all assets."
resim set-default-account $account_address_1 $private_key_1 $owner_badge_1":"$owner_badge_id_1

echo -e "\n16. Exchange Assets."

output=$(account_1=${account_address_1} \
  ticket=${ticket} \
  escrow_component=${escrow_component_1} \
  badge=${escrow_badge_1} \
  id="#1#" \
  swap_contract=${swap_component_1} \
  resim run manifest/offerable_swap/07_select_offer_and_exchange_assets.rtm | grep -o "Transaction.*")

echo $output 

resim show $account_address_1

echo -e "\n17. Set default account to address 2 and and take all assets from contract."
resim set-default-account $account_address_2 $private_key_2 $owner_badge_2":"$owner_badge_id_2

echo -e "\n18. Withdraw Assets."

output=$(account_1=${account_address_2} \
  ticket=${ticket} \
  escrow_component=${escrow_component_1} \
  badge=${escrow_badge_1} \
  id="#2#" \
  swap_contract=${swap_component_1} \
  resim run manifest/offerable_swap/08_withdraw_assets.rtm | grep -o "Transaction.*")

echo $output 

resim show $account_address_2