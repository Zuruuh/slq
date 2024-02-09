create database other_database;
create database slq;

create table slq.users (
    id int not null auto_increment primary key,
    username varchar(32) not null,
    email varchar(255) not null,
    password varchar(255) not null,
    registered_at timestamp default current_timestamp,
    unique(username),
    unique(email)
);

insert into slq.users (username, email, password) values
('john_doe', 'john@example.com', 'password123'),
('jane_smith', 'jane@example.com', 'secret456'),
('mike_jackson', 'mike@example.com', 'mysecurepassword');

create table slq.posts (
    id int not null auto_increment primary key,
    author_id int not null,
    title varchar(255) not null,
    content text not null,
    posted_at timestamp default current_timestamp,
    foreign key (author_id) references users(id)
);

create user 'super_admin'@'%' identified with caching_sha2_password by 'passwd';
create user 'admin'@'%' identified with caching_sha2_password by 'passwd';
create user 'reader'@'%' identified with caching_sha2_password by 'passwd';
create user 'inserter'@'%' identified with caching_sha2_password by 'passwd';
create user 'updater'@'%' identified with caching_sha2_password by 'passwd';
create user 'deleter'@'%' identified with caching_sha2_password by 'passwd';
create user 'writer'@'%' identified with caching_sha2_password by 'passwd';

grant all on *.* to 'super_admin'@'%';
grant all on slq.* to 'admin'@'%';
grant select on slq.* to 'reader'@'%';
grant select, insert on slq.* to 'inserter'@'%';
grant select, delete on slq.* to 'deleter'@'%';
grant select, update on slq.* to 'updater'@'%';
grant select, insert, delete, update on slq.* to 'writer'@'%';
