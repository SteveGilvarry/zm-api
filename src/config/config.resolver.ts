import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ConfigService } from './config.service';
import { ConfigUpdateInput } from '../@generated/prisma-nestjs-graphql/config/config-update.input';
import { ConfigCreateInput} from '../@generated/prisma-nestjs-graphql/config/config-create.input';
import { ConfigWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/config/config-where-unique.input';


@Resolver('Config')
export class ConfigResolver {
  constructor(private readonly configService: ConfigService) {}

  @Mutation('createConfig')
  create(@Args('createConfigInput') createConfigInput: ConfigCreateInput) {
    return this.configService.create(createConfigInput);
  }

  @Query('configs')
  findAll() {
    return this.configService.findAll();
  }

  @Query('config')
  findOne(@Args('name') Name: string) {
    return this.configService.findOne({ Name });
  }

  @Mutation('updateConfig')
  update(
    @Args( 'name') Name: string,
    @Args('updateConfigInput') updateConfigInput: ConfigUpdateInput) {
    return this.configService.update({ Name }, updateConfigInput);
  }

  @Mutation('removeConfig')
  remove(@Args('id') id: number) {
    return this.configService.remove(id);
  }
}
