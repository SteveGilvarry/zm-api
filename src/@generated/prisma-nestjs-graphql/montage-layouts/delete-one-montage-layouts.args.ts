import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:false})
    @Type(() => MontageLayoutsWhereUniqueInput)
    where!: MontageLayoutsWhereUniqueInput;
}
