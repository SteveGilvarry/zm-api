import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { UsersService } from './users.service';
import { UsersWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/users/users-where-unique.input';
import { UsersUpdateInput } from '../@generated/prisma-nestjs-graphql/users/users-update.input';
import { UsersCreateInput } from '../@generated/prisma-nestjs-graphql/users/users-create.input';

@Resolver('User')
export class UsersResolver {
  constructor(private readonly usersService: UsersService) {}

  @Mutation('createUser')
  create(@Args('createUserInput') createUserInput: UsersCreateInput) {
    return this.usersService.create(createUserInput);
  }

  @Query('users')
  findAll() {
    return this.usersService.findAll();
  }

  @Query('user')
  findOne(@Args('username') username: UsersWhereUniqueInput) {
    return this.usersService.findOne(username);
  }

  @Mutation('updateUser')
  update(
    @Args('usersWhereUniqueInput') usersWhereUniqueInput: UsersWhereUniqueInput,
    @Args('updateUserInput') updateUserInput: UsersUpdateInput,
  ) {
    return this.usersService.update(usersWhereUniqueInput, updateUserInput);
  }

  @Mutation('removeUser')
  remove(@Args('id') id: number) {
    return this.usersService.remove(id);
  }
}
