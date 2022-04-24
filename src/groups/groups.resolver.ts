import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { GroupsService } from './groups.service';
import { GroupsCreateInput} from '../@generated/prisma-nestjs-graphql/groups/groups-create.input';
import { GroupsUpdateInput} from '../@generated/prisma-nestjs-graphql/groups/groups-update.input';
import { GroupsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/groups/groups-where-unique.input';

@Resolver('Group')
export class GroupsResolver {
  constructor(private readonly groupsService: GroupsService) {}

  @Mutation('createGroup')
  async create(
    @Args('createGroupInput') createGroupInput: GroupsCreateInput
  ) {
    const created = await this.groupsService.create(createGroupInput);
  }

  @Query('groups')
  findAll() {
    return this.groupsService.findAll();
  }

  @Query('group')
  findOne(@Args('id') Id: number) {
    return this.groupsService.findOne( {Id} );
  }

  @Mutation('updateGroup')
  update(
    @Args('id') Id: number,
    @Args('updateGroupInput') updateGroupInput: GroupsUpdateInput
  ) {
    return this.groupsService.update( {Id}, updateGroupInput);
  }

  @Mutation('removeGroup')
  remove(@Args('id') Id: number) {
    return this.groupsService.remove( {Id});
  }
}
