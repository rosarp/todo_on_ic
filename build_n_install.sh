dfx start --clean --background
dfx canister create todo_on_ic_backend
dfx build todo_on_ic_backend
dfx generate
dfx canister install todo_on_ic_backend
