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

echo -e "\n3. Instansiate Escrow package."
output=$(account_1=${account_address_1} \
  escrow_package_address=${escrow_package} \
  resim run manifest/instantiate.rtm)
escrow_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
escrow_badge_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*")
resim show $escrow_component_1
resim show $escrow_badge_1

echo -e "\n4. Create Assets to trade."
output=$(account_1=${account_address_1} \
  resim run manifest/create_nft.rtm)
nft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to offer : "$nft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/create_nft.rtm)
nft_resource_address_2=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "NFT to require : "$nft_resource_address_2

output=$(account_1=${account_address_1} \
  resim run manifest/create_ft.rtm)
ft_resource_address_1=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to offer : "$ft_resource_address_1

output=$(account_1=${account_address_1} \
  resim run manifest/create_ft.rtm)
ft_resource_address_2=$(echo "$output" | grep "Resource:" | grep -o "resource_.*" | awk 'END{$1=$1; print}')
echo "Token to require : "$ft_resource_address_2

echo -e "\n5. Create Account 2."
output=$(resim new-account)

account_address_2=$(echo "$output" | grep "Account" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
public_key_2=$(echo "$output" | grep "Public key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
private_key_2=$(echo "$output" | grep "Private key" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $2}' | awk '{$1=$1; print}')
owner_badge_id_2=$(echo "$output" | grep "Owner badge" | awk -F':' '{print $3}' | awk '{$1=$1; print}')
echo "Account address 2 : "$account_address_2

echo -e "\n6. Transfering to required assets to account 2."
output=$(account_1=${account_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_2=${nft_resource_address_2} \
  account_2=${account_address_2} \
  resim run manifest/transfer_assets.rtm)
echo "Account 2 state :"
resim show $account_address_2

echo -e "\n7. Create Swap Contract."
output=$(account_1=${account_address_1} \
  ft_resource_1=${ft_resource_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_1=${nft_resource_address_1} \
  nft_resource_2=${nft_resource_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/create_swap.rtm)
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n8. Cancel contract."
output=$(account_1=${account_address_1} \
  badge_address=${escrow_badge_1} \
  id="#0#" \
  swap_contract=${swap_component_1} \
  resim run manifest/cancel_swap.rtm)
echo $swap_component_1 "Cancelled."
resim show

echo -e "\n9. re-Create Swap Contract."
output=$(account_1=${account_address_1} \
  ft_resource_1=${ft_resource_address_1} \
  ft_resource_2=${ft_resource_address_2} \
  nft_resource_1=${nft_resource_address_1} \
  nft_resource_2=${nft_resource_address_2} \
  escrow_component=${escrow_component_1} \
  resim run manifest/create_swap.rtm)
swap_component_1=$(echo "$output" | grep "Component:" | grep -o "component_.*")
resim show $swap_component_1

echo -e "\n10. Withdrawing assets (error intended)."
output=$(account_1=${account_address_1} \
  badge_address=${escrow_badge_1} \
  id="#1#" \
  swap_component=${swap_component_1} \
  resim run manifest/withdraw_assets.rtm || true)

echo -e "\n11. Set default address to account 2."
resim set-default-account $account_address_2 $private_key_2 $owner_badge_2":"$owner_badge_id_2

echo -e "\n12. Exchange assets."
output=$(account_2=${account_address_2} \
  nft_resource_2=${nft_resource_address_2} \
  ft_resource_2=${ft_resource_address_2} \
  swap_component=${swap_component_1} \
  resim run manifest/exchange_assets.rtm | grep -o "Transaction.*")

echo $output 

resim show $account_address_2

echo -e "\n13. Set default account to address 1 and withdraw all assets."
resim set-default-account $account_address_1 $private_key_1 $owner_badge_1":"$owner_badge_id_1
output=$(account_1=${account_address_1} \
  badge_address=${escrow_badge_1} \
  id="#1#" \
  swap_component=${swap_component_1} \
  resim run manifest/withdraw_assets.rtm | grep -o "Transaction.*")

echo $output 

resim show $account_address_1
