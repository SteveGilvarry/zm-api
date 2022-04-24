import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';

@ArgsType()
export class FindUniqueZonesArgs {

    @Field(() => ZonesWhereUniqueInput, {nullable:false})
    where!: ZonesWhereUniqueInput;
}
