create schema auth;

create table auth.users (
    id uuid primary key default uuid_generate_v4(),
    user_role text not null,
    email text not null unique,
    password_hash text not null
);

create table auth.refresh_tokens (
    id uuid primary key default uuid_generate_v4(),
    user_id uuid not null references auth.users (id) on delete cascade,
    expires_at timestamp with time zone not null,
    created_at timestamp with time zone not null default timezone('utc', now())
);


create table auth.password_reset_tokens (
    id uuid primary key default uuid_generate_v4(),
    user_id uuid not null references auth.users (id) on delete cascade,
    expires_at timestamp with time zone not null
);
