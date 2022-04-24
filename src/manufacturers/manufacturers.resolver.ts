import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ManufacturersService } from './manufacturers.service';
import { ManufacturersCreateInput} from '../@generated/prisma-nestjs-graphql/manufacturers/manufacturers-create.input';
import { ManufacturersUpdateInput} from '../@generated/prisma-nestjs-graphql/manufacturers/manufacturers-update.input';
import { ManufacturersWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/manufacturers/manufacturers-where-unique.input';

@Resolver('Manufacturer')
export class ManufacturersResolver {
  constructor(private readonly manufacturersService: ManufacturersService) {}

  @Mutation('createManufacturer')
  async create(
    @Args('createManufacturerInput') createManufacturerInput: ManufacturersCreateInput
  ) {
    const created = await this.manufacturersService.create(createManufacturerInput);
  }

  @Query('manufacturers')
  findAll() {
    return this.manufacturersService.findAll();
  }

  @Query('manufacturer')
  findOne(@Args('id') Id: number) {
    return this.manufacturersService.findOne({ Id });
  }

  @Mutation('updateManufacturer')
  update(
    @Args('id') Id: number,
    @Args('updateManufacturerInput') updateManufacturerInput: ManufacturersUpdateInput) {
    return this.manufacturersService.update({ Id }, updateManufacturerInput);
  }

  @Mutation('removeManufacturer')
  remove(@Args('id') Id: number) {
    return this.manufacturersService.remove({ Id });
  }
}
