import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';
import { ZonesCreateInput } from './zones-create.input';
import { ZonesUpdateInput } from './zones-update.input';

@ArgsType()
export class UpsertOneZonesArgs {

    @Field(() => ZonesWhereUniqueInput, {nullable:false})
    where!: ZonesWhereUniqueInput;

    @Field(() => ZonesCreateInput, {nullable:false})
    create!: ZonesCreateInput;

    @Field(() => ZonesUpdateInput, {nullable:false})
    update!: ZonesUpdateInput;
}
