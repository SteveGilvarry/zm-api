import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { FiltersService } from './filters.service';
import { FiltersCreateInput} from '../@generated/prisma-nestjs-graphql/filters/filters-create.input';
import { FiltersUpdateInput} from '../@generated/prisma-nestjs-graphql/filters/filters-update.input';
import { FiltersWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/filters/filters-where-unique.input';
import { CreateFilterInput } from '../graphql';


@Resolver('Filter')
export class FiltersResolver {
  constructor(private readonly filtersService: FiltersService) {}

  @Mutation('createFilter')
  async create(
    @Args('createFilterInput') createFilterInput: FiltersCreateInput
  ) {
    const created = await this.filtersService.create(createFilterInput);
  }

  @Query('filters')
  findAll() {
    return this.filtersService.findAll();
  }

  @Query('filter')
  findOne(@Args('id') Id: number) {
    return this.filtersService.findOne( { Id });
  }

  @Mutation('updateFilter')
  update(
    @Args('id') Id: number,
    @Args('updateFilterInput') updateFilterInput: FiltersUpdateInput) {
    return this.filtersService.update({ Id }, updateFilterInput);
  }

  @Mutation('removeFilter')
  remove(@Args('id') Id: number) {
    return this.filtersService.remove({Id} );
  }
}
