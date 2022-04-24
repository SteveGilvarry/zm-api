import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereInput } from './zones-where.input';

@ArgsType()
export class DeleteManyZonesArgs {

    @Field(() => ZonesWhereInput, {nullable:true})
    where?: ZonesWhereInput;
}
