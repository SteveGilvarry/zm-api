import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneZonesArgs {

    @Field(() => ZonesWhereUniqueInput, {nullable:false})
    @Type(() => ZonesWhereUniqueInput)
    where!: ZonesWhereUniqueInput;
}
