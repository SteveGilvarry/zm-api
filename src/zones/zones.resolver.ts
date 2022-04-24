import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ZonesService } from './zones.service';
import { ZonesCreateInput } from '../@generated/prisma-nestjs-graphql/zones/zones-create.input';
import { ZonesUpdateInput } from '../@generated/prisma-nestjs-graphql/zones/zones-update.input';
import { ZonesWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/zones/zones-where-unique.input';


@Resolver('Zone')
export class ZonesResolver {
  constructor(private readonly zonesService: ZonesService) {}

  @Mutation('createZone')
  async create(
    @Args('createZoneInput') createZoneInput: ZonesCreateInput
  ) {
    const created = await this.zonesService.create(createZoneInput);
  }

  @Query('zones')
  findAll() {
    return this.zonesService.findAll();
  }

  @Query('zone')
  findOne(@Args('id') Id: number) {
    return this.zonesService.findOne( { Id });
  }

  @Mutation('updateZone')
  update(
    @Args('id') Id: number,
    @Args('updateZoneInput') updateZoneInput: ZonesUpdateInput) {
    return this.zonesService.update( { Id }, updateZoneInput);
  }

  @Mutation('removeZone')
  remove(@Args('id') Id: number) {
    return this.zonesService.remove( { Id });
  }
}
