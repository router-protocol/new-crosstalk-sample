echo "current directory"
echo $PWD


cd ../../routerswap/deployment
echo "current directory"
echo $PWD

npx ts-node scripts/init_dex.ts 

cd -
echo "current directory"
echo $PWD

echo "copying the dex.json file"
cp ../../routerswap/deployment/config/dex.json config

npx ts-node scripts/init_texchange.ts 
npx ts-node scripts/set_initial_configurations.ts
npx ts-node scripts/provide_liquidity.ts
