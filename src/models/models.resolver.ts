import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ModelsService } from './models.service';
import { ModelsCreateInput} from '../@generated/prisma-nestjs-graphql/models/models-create.input';
import { ModelsUpdateInput} from '../@generated/prisma-nestjs-graphql/models/models-update.input';
import { ModelsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/models/models-where-unique.input';

@Resolver('Model')
export class ModelsResolver {
  constructor(private readonly modelsService: ModelsService) {}

  @Mutation('createModel')
  async create(
    @Args('createModelInput') createModelInput: ModelsCreateInput,
  ) {
    const created = await this.modelsService.create(createModelInput);
  }

  @Query('models')
  findAll() {
    return this.modelsService.findAll();
  }

  @Query('model')
  findOne(@Args('id') Id: number) {
    return this.modelsService.findOne( {Id} );
  }

  @Mutation('updateModel')
  update(
    @Args('id') Id: number,
    @Args('updateModelInput') updateModelInput: ModelsUpdateInput) {
    return this.modelsService.update( { Id }, updateModelInput);
  }

  @Mutation('removeModel')
  remove(@Args('id') Id: number) {
    return this.modelsService.remove( { Id } );
  }
}
