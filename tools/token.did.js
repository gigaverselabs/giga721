export const idlFactory = ({ IDL }) => {
  const Result = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Property = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const TokenDesc = IDL.Record({
    'id' : IDL.Nat,
    'url' : IDL.Text,
    'owner' : IDL.Principal,
    'desc' : IDL.Text,
    'name' : IDL.Text,
    'properties' : IDL.Vec(Property),
  });
  const HeaderField = IDL.Tuple(IDL.Text, IDL.Text);
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
    'status_code' : IDL.Nat16,
  });
  const Time = IDL.Int;
  const Listing = IDL.Record({
    'token_id' : IDL.Nat,
    'owner' : IDL.Principal,
    'time' : Time,
    'price' : IDL.Nat64,
  });
  const ICPTs = IDL.Record({ 'e8s' : IDL.Nat64 });
  const TransactionNotification = IDL.Record({
    'to' : IDL.Principal,
    'to_subaccount' : IDL.Opt(IDL.Nat8),
    'from' : IDL.Principal,
    'memo' : IDL.Nat64,
    'from_subaccount' : IDL.Opt(IDL.Nat8),
    'amount' : ICPTs,
    'block_height' : IDL.Nat64,
  });
  const Asset = IDL.Record({
    'contentType' : IDL.Text,
    'data' : IDL.Vec(IDL.Nat8),
    'name' : IDL.Text,
    'properties' : IDL.Vec(Property),
  });
  const Result2 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const Token = IDL.Record({
    'id' : IDL.Nat,
    'url' : IDL.Text,
    'desc' : IDL.Text,
    'name' : IDL.Text,
    'properties' : IDL.Vec(Property),
  });
  return IDL.Service({
    'add_genesis_record' : IDL.Func([], [IDL.Nat], []),
    'burn' : IDL.Func([IDL.Nat], [Result], []),
    'creators_fee' : IDL.Func([], [IDL.Nat], ['query']),
    'data_of' : IDL.Func([IDL.Nat], [TokenDesc], ['query']),
    'delist' : IDL.Func([IDL.Nat], [Result], []),
    'description' : IDL.Func([], [IDL.Text], ['query']),
    'get_cycles' : IDL.Func([], [IDL.Nat], ['query']),
    'get_ledger_canister' : IDL.Func([], [IDL.Opt(IDL.Principal)], ['query']),
    'get_listed_count' : IDL.Func([], [IDL.Nat], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'icon_url' : IDL.Func([], [IDL.Text], ['query']),
    'is_paused' : IDL.Func([], [IDL.Bool], ['query']),
    'list' : IDL.Func([IDL.Nat, IDL.Nat64], [Result], []),
    'listings' : IDL.Func([], [IDL.Vec(Listing)], ['query']),
    'mint_for' : IDL.Func([IDL.Nat, IDL.Principal], [Result], []),
    'name' : IDL.Func([], [IDL.Text], ['query']),
    'owner' : IDL.Func([], [IDL.Principal], ['query']),
    'owner_of' : IDL.Func([IDL.Nat], [IDL.Principal], ['query']),
    'set_description' : IDL.Func([IDL.Text], [IDL.Bool], []),
    'set_icon_url' : IDL.Func([IDL.Text], [IDL.Bool], []),
    'set_ledger_canister' : IDL.Func([IDL.Principal], [IDL.Bool], []),
    'set_owner' : IDL.Func([IDL.Principal], [IDL.Bool], []),
    'set_paused' : IDL.Func([IDL.Bool], [IDL.Bool], []),
    'set_tx_enabled' : IDL.Func([IDL.Bool], [IDL.Bool], []),
    'symbol' : IDL.Func([], [IDL.Text], ['query']),
    'tokens' : IDL.Func([], [IDL.Vec(TokenDesc)], ['query']),
    'total_supply' : IDL.Func([], [IDL.Nat], ['query']),
    'transaction_notification' : IDL.Func(
        [TransactionNotification],
        [Result],
        [],
      ),
    'transfer_to' : IDL.Func([IDL.Principal, IDL.Nat], [IDL.Bool], []),
    'tx_enabled' : IDL.Func([], [IDL.Bool], ['query']),
    'upload_asset' : IDL.Func([Asset], [Result2], []),
    'upload_tokens_metadata' : IDL.Func([IDL.Vec(Token)], [Result2], []),
    'user_tokens' : IDL.Func([IDL.Principal], [IDL.Vec(IDL.Nat)], ['query']),
  });
};
export const init = ({ IDL }) => {
  return [IDL.Text, IDL.Text, IDL.Text, IDL.Nat, IDL.Principal];
};
