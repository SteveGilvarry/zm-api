import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { MontagelayoutsService } from './montagelayouts.service';
import { MontageLayoutsCreateInput} from '../@generated/prisma-nestjs-graphql/montage-layouts/montage-layouts-create.input';
import { MontageLayoutsUpdateInput} from '../@generated/prisma-nestjs-graphql/montage-layouts/montage-layouts-update.input';
import { MontageLayoutsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/montage-layouts/montage-layouts-where-unique.input';

@Resolver('Montagelayout')
export class MontagelayoutsResolver {
  constructor(private readonly montagelayoutsService: MontagelayoutsService) {}

  @Mutation('createMontagelayout')
  async create(
    @Args('createMontagelayoutInput') createMontagelayoutInput: MontageLayoutsCreateInput
  ) {
    const created = await this.montagelayoutsService.create(createMontagelayoutInput);
  }

  @Query('montagelayouts')
  findAll() {
    return this.montagelayoutsService.findAll();
  }

  @Query('montagelayout')
  findOne(@Args('id') Id: number) {
    return this.montagelayoutsService.findOne({ Id });
  }

  @Mutation('updateMontagelayout')
  update(
    @Args('id') Id: number,
    @Args('updateMontagelayoutInput') updateMontagelayoutInput: MontageLayoutsUpdateInput
  ) {
    return this.montagelayoutsService.update({ Id }, updateMontagelayoutInput);
  }

  @Mutation('removeMontagelayout')
  remove(@Args('id') Id: number) {
    return this.montagelayoutsService.remove({ Id });
  }
}
