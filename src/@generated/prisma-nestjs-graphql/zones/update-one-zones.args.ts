import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesUpdateInput } from './zones-update.input';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';

@ArgsType()
export class UpdateOneZonesArgs {

    @Field(() => ZonesUpdateInput, {nullable:false})
    data!: ZonesUpdateInput;

    @Field(() => ZonesWhereUniqueInput, {nullable:false})
    where!: ZonesWhereUniqueInput;
}
