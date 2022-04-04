dfx stop
dfx start --clean --background
./local_deploy.sh
cd tools
node 2_upload_token_metadata.js
cd ..