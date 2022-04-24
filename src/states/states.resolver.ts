import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { StatesService } from './states.service';
import {StatesCreateInput} from '../@generated/prisma-nestjs-graphql/states/states-create.input';
import {StatesUpdateInput} from '../@generated/prisma-nestjs-graphql/states/states-update.input';
import {StatesWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/states/states-where-unique.input';

@Resolver('State')
export class StatesResolver {
  constructor(private readonly statesService: StatesService) {}

  @Mutation('createState')
  async create(
    @Args('createStateInput') createStateInput: StatesCreateInput
  ) {
    const created = await this.statesService.create(createStateInput);
  }

  @Query('states')
  findAll() {
    return this.statesService.findAll();
  }

  @Query('state')
  findOne(@Args('id') Id: number) {
    return this.statesService.findOne({Id});
  }

  @Mutation('updateState')
  update(
    @Args('id') Id: number,
    @Args('updateStateInput') updateStateInput: StatesUpdateInput) {
    return this.statesService.update({ Id }, updateStateInput);
  }

  @Mutation('removeState')
  remove(@Args('id') Id: number) {
    return this.statesService.remove( { Id } );
  }
}
