import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ServersService } from './servers.service';
import { ServersCreateInput} from '../@generated/prisma-nestjs-graphql/servers/servers-create.input';
import { ServersUpdateInput} from '../@generated/prisma-nestjs-graphql/servers/servers-update.input';
import { ServersWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/servers/servers-where-unique.input';


@Resolver('Server')
export class ServersResolver {
  constructor(private readonly serversService: ServersService) {}

  @Mutation('createServer')
  async create(
    @Args('createServerInput') createServerInput: ServersCreateInput
  ) {
    const created = await this.serversService.create(createServerInput);
  }

  @Query('servers')
  findAll() {
    return this.serversService.findAll();
  }

  @Query('server')
  findOne(@Args('id') Id: number) {
    return this.serversService.findOne( { Id } );
  }

  @Mutation('updateServer')
  update(
    @Args('id') Id: number,
    @Args('updateServerInput') updateServerInput: ServersUpdateInput) {
    return this.serversService.update({ Id }, updateServerInput);
  }

  @Mutation('removeServer')
  remove(@Args('id') Id: number) {
    return this.serversService.remove({ Id });
  }
}
