create database other_database;
create database slq;

create table slq.slq_data (
    id int not null auto_increment,
    primary key (id)
);

create table slq.slq_other_data (
    id int not null auto_increment,
    primary key (id)
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
