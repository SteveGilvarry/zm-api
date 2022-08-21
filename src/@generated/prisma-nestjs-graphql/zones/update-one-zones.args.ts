import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesUpdateInput } from './zones-update.input';
import { Type } from 'class-transformer';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';

@ArgsType()
export class UpdateOneZonesArgs {

    @Field(() => ZonesUpdateInput, {nullable:false})
    @Type(() => ZonesUpdateInput)
    data!: ZonesUpdateInput;

    @Field(() => ZonesWhereUniqueInput, {nullable:false})
    @Type(() => ZonesWhereUniqueInput)
    where!: ZonesWhereUniqueInput;
}
