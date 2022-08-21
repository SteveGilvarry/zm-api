import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';
import { Type } from 'class-transformer';
import { ZonesCreateInput } from './zones-create.input';
import { ZonesUpdateInput } from './zones-update.input';

@ArgsType()
export class UpsertOneZonesArgs {

    @Field(() => ZonesWhereUniqueInput, {nullable:false})
    @Type(() => ZonesWhereUniqueInput)
    where!: ZonesWhereUniqueInput;

    @Field(() => ZonesCreateInput, {nullable:false})
    @Type(() => ZonesCreateInput)
    create!: ZonesCreateInput;

    @Field(() => ZonesUpdateInput, {nullable:false})
    @Type(() => ZonesUpdateInput)
    update!: ZonesUpdateInput;
}
