-- Add migration script here
create extension if not exists "uuid-ossp"; -- noqa: RF05
create schema if not exists auth;
create schema if not exists ifq_charters;
create schema if not exists uploads;
